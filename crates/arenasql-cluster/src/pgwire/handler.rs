use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use arenasql::ast::statement::StatementType;
use arenasql::bytes::Bytes;
use arenasql::datafusion::{LogicalPlan, ScalarValue};
use async_trait::async_trait;
use futures::{Sink, SinkExt};
use itertools::Itertools;
use nom::AsBytes;
use pgwire::api::portal::Portal;
use pgwire::api::query::{
  ExtendedQueryHandler, SimpleQueryHandler, StatementOrPortal,
};
use pgwire::api::results::{DescribeResponse, FieldInfo, Response};
use pgwire::api::stmt::QueryParser;
use pgwire::api::store::PortalStore;
use pgwire::api::{ClientInfo, ClientPortalStore, Type};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use pgwire::messages::extendedquery::{Bind, BindComplete};
use pgwire::messages::PgWireBackendMessage;

use super::portal::ArenaPortalState;
use super::{ArenaQuery, ArenaQueryParser};
use crate::pgwire::datatype;
use crate::server::ArenaSqlCluster;

#[async_trait]
impl ExtendedQueryHandler for ArenaSqlCluster {
  type PortalState = ArenaPortalState;
  type Statement = ArenaQuery;
  type QueryParser = ArenaQueryParser;

  /// Prepares the logical plan for the query and bind the parameters to it
  #[tracing::instrument(skip(self, client), level = "trace")]
  async fn on_bind<C>(&self, client: &mut C, message: Bind) -> PgWireResult<()>
  where
    C: ClientInfo
      + ClientPortalStore
      + Sink<PgWireBackendMessage>
      + Unpin
      + Send
      + Sync,
    C::Error: Debug,
    C::PortalStore:
      PortalStore<Statement = Self::Statement, State = Self::PortalState>,
    PgWireError: From<<C as Sink<PgWireBackendMessage>>::Error>,
  {
    let session = self.get_client_session(client)?;
    let txn = session.create_transaction()?;

    let statement_name = message
      .statement_name()
      .as_deref()
      .unwrap_or(pgwire::api::DEFAULT_NAME);

    if let Some(statement) = client.portal_store().get_statement(statement_name)
    {
      let query = statement.statement();
      // If the query planning was successful, add the plan to the portal
      // state. It could fail if the placeholder type can't be resolved just
      // from the query itself and needs the paramter values as well
      let state = match txn
        .create_verified_logical_plan(query.stmts[0].clone())
        .await
      {
        Ok(plan) => {
          let (params, fields) = get_params_and_field_types(&plan)?;
          Some(
            ArenaPortalState::default()
              .set_query_plan(Some(plan))
              .set_params(params)
              .set_fields(fields)
              .to_owned(),
          )
        }
        _ => None,
      };

      let portal = Portal::try_new(&message, statement, state)?;
      client.portal_store().put_portal(Arc::new(portal));
      client
        .send(PgWireBackendMessage::BindComplete(BindComplete::new()))
        .await?;
      Ok(())
    } else {
      Err(PgWireError::StatementNotFound(statement_name.to_owned()))
    }
  }

  #[tracing::instrument(skip_all, level = "trace")]
  async fn do_query<'p, 'h: 'p, C>(
    &'h self,
    client: &mut C,
    portal: &'p Portal<ArenaQuery, ArenaPortalState>,
    _max_rows: usize,
  ) -> PgWireResult<Response<'p>>
  where
    C: ClientInfo + Send,
  {
    let session = self.get_client_session(client)?;
    let (transaction, chained) = session.get_active_transaction().map_or_else(
      || session.create_transaction().map(|t| (t, false)),
      |txn| Ok((txn.clone(), true)),
    )?;
    let stmts = &portal.statement().statement().stmts;
    let stmt = stmts[0].clone();
    let params = portal.parameters();

    let prams_values = params
      .iter()
      .zip::<&Vec<Type>>(
        portal
          .state()
          .as_ref()
          .expect("Portal state to found")
          .params(),
      )
      .map(|(param, r#type)| {
        convert_bytes_to_scalar_value(param.as_ref(), r#type)
      })
      .collect::<PgWireResult<Vec<ScalarValue>>>()?;

    // Use existing plan if the portal has it
    let plan =
      match portal.state().as_ref().and_then(|s| s.query_plan().clone()) {
        Some(plan) => plan,
        None => {
          transaction
            .create_verified_logical_plan(stmt.clone())
            .await?
        }
      };

    // TODO: remove this
    log::trace!("do_query raw params: = {:#?}", params);
    log::trace!("do_query param values: = {:?}", prams_values);
    let final_plan = plan
      .with_param_values(prams_values)
      .map_err(|e| arenasql::Error::DataFusionError(e.into()))
      .expect(&format!(
        "Error replace_params_with_values at: {}:{}",
        file!(),
        line!()
      ));

    let stmt_type = StatementType::from(stmt.as_ref());
    let response = transaction
      .execute_logical_plan(&stmt_type, final_plan)
      .await?;
    // Commit the transaction if it's not a chained transaction
    // i.e. if it wasn't explicitly started by `BEGIN` command
    let transaction_to_commit = if !chained { Some(transaction) } else { None };

    Self::map_to_pgwire_response(&stmt_type, response, transaction_to_commit)
      .await
  }

  #[tracing::instrument(skip_all, level = "trace")]
  fn query_parser(&self) -> Arc<Self::QueryParser> {
    self.parser.clone()
  }

  #[tracing::instrument(skip_all, level = "trace")]
  async fn do_describe<C>(
    &self,
    client: &mut C,
    target: StatementOrPortal<'_, Self::Statement, Self::PortalState>,
  ) -> PgWireResult<DescribeResponse>
  where
    C: ClientInfo + Send,
  {
    let (maybe_plan, stmt) = match target {
      StatementOrPortal::Portal(portal) => (
        portal.state().as_ref().and_then(|s| s.query_plan().clone()),
        portal.statement().as_ref(),
      ),
      StatementOrPortal::Statement(stmt) => (None, stmt),
    };

    let plan = match maybe_plan {
      Some(plan) => plan,
      None => {
        let session = self.get_client_session(client)?;
        let txn = session.create_transaction()?;
        txn
          .create_verified_logical_plan(stmt.statement().stmts[0].clone())
          .await?
      }
    };
    let (params, fields) = get_params_and_field_types(&plan)?;
    Ok(DescribeResponse::new(Some(params), fields))
  }
}

#[async_trait]
impl SimpleQueryHandler for ArenaSqlCluster {
  #[tracing::instrument(skip_all, level = "trace")]
  async fn do_query<'a, C>(
    &self,
    client: &mut C,
    query: &'a str,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo + Unpin + Send + Sync,
  {
    let parsed_query = self.parser.parse_sql(query, &[Type::ANY]).await?;
    let results_fut = self.execute_query(client, parsed_query);

    match results_fut.await {
      Ok(response) => Ok(response),
      Err(e) => Ok(vec![Response::Error(Box::new(ErrorInfo::new(
        "ERROR".to_owned(),
        "XX000".to_owned(),
        e.to_string(),
      )))]),
    }
  }
}

fn get_params_and_field_types(
  plan: &LogicalPlan,
) -> PgWireResult<(Vec<Type>, Vec<FieldInfo>)> {
  // Expects placeholder to be in format "${index}"
  let params = plan
    .get_parameter_types()
    .unwrap()
    .iter()
    .map(|(id, r#type)| {
      let index = id[1..]
        .parse::<usize>()
        .expect(&format!("Error parsing param index: {:?}", id));
      (
        index,
        r#type
          .as_ref()
          .map(|t| datatype::derive_pg_type(&t, &HashMap::new()))
          .unwrap_or(Type::TEXT),
      )
    })
    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
    .map(|(_, t)| t)
    .collect();

  let field = plan
    .schema()
    .fields()
    .iter()
    .map(|f| datatype::to_field_info(f.field().as_ref()))
    .collect();

  Ok((params, field))
}

fn convert_bytes_to_scalar_value(
  bytes: Option<&Bytes>,
  r#type: &Type,
) -> PgWireResult<ScalarValue> {
  let scalar = match *r#type {
    Type::BOOL => {
      ScalarValue::Boolean(bytes.map(|b| if b[0] > 0 { true } else { false }))
    }
    Type::INT4 => ScalarValue::Int32(bytes.as_ref().and_then(|b| {
      Some(i32::from_be_bytes(b.as_bytes().try_into().unwrap()))
    })),
    Type::INT8 => ScalarValue::Int64(bytes.as_ref().and_then(|b| {
      Some(i64::from_be_bytes(b.as_bytes().try_into().unwrap()))
    })),
    Type::TEXT | Type::VARCHAR => ScalarValue::Utf8(
      bytes.and_then(|b| std::str::from_utf8(&b).map(|s| s.to_owned()).ok()),
    ),
    _ => {
      unimplemented!("Converting bytes to ScalarValue for type {:?}", r#type)
    }
  };

  Ok(scalar)
}
