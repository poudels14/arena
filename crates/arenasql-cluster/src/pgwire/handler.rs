use std::fmt::Debug;
use std::sync::Arc;

use arenasql::arrow::{Float32Type, ListArray};
use arenasql::bytes::Bytes;
use arenasql::datafusion::{LogicalPlan, ScalarValue};
use arenasql::pgwire::api::portal::{Format, Portal};
use arenasql::pgwire::api::query::{
  ExtendedQueryHandler, SimpleQueryHandler, StatementOrPortal,
};
use arenasql::pgwire::api::results::{
  DescribeResponse, FieldFormat, FieldInfo, Response,
};
use arenasql::pgwire::api::stmt::{QueryParser, StoredStatement};
use arenasql::pgwire::api::store::PortalStore;
use arenasql::pgwire::api::{ClientInfo, ClientPortalStore, Type};
use arenasql::pgwire::error::{PgWireError, PgWireResult};
use arenasql::pgwire::messages::extendedquery::{
  Bind, BindComplete, Parse, ParseComplete,
};
use arenasql::pgwire::messages::PgWireBackendMessage;
use arenasql::postgres_types::FromSql;
use arenasql::schema::CTID_COLUMN;
use arenasql::{pgwire, Error};
use async_trait::async_trait;
use futures::{Sink, SinkExt};
use itertools::Itertools;
use nom::AsBytes;

use super::portal::ArenaPortalState;
use super::{ArenaQuery, ArenaQueryParser};
use crate::auth::AuthHeader;
use crate::pgwire::datatype;
use crate::server::ArenaSqlCluster;

#[async_trait]
impl ExtendedQueryHandler for ArenaSqlCluster {
  type PortalState = ArenaPortalState;
  type Statement = ArenaQuery;
  type QueryParser = ArenaQueryParser;

  #[tracing::instrument(skip(self, client), level = "trace")]
  async fn on_parse<C>(
    &self,
    client: &mut C,
    message: Parse,
  ) -> PgWireResult<()>
  where
    C: ClientInfo
      + ClientPortalStore
      + Sink<PgWireBackendMessage>
      + Unpin
      + Send
      + Sync,
    C::PortalStore: PortalStore<Statement = Self::Statement>,
    C::Error: Debug,
    PgWireError: From<<C as Sink<PgWireBackendMessage>>::Error>,
  {
    let parser = Arc::new(ArenaQueryParser {});
    let stmt = StoredStatement::parse(&message, parser).await?;
    let statement = stmt.statement.clone();

    // From Postgres doc:
    // The query string contained in a Parse message cannot include more than
    // one SQL statement; else a syntax error is reported.
    if statement.stmts.len() > 1 {
      return Err(
        Error::InvalidQuery(format!("More than one SQL statement not allowed"))
          .into(),
      );
    }

    let session = match &statement.client {
      AuthHeader::None => self.get_client_session(client),
      header => self.get_or_create_new_session(client, &header),
    }?;

    // Note: create verified plan to make sure query is valid.
    // Query could be invalid if it uses table that doesn't exits, etc
    // TODO: for a single prepared statement, verified logical plan is
    // created in several stages. Figure out a way to minimize the number
    // of times verified logical plan is created. Maybe just check for
    // catalog/schema/table/column relations during parse instead of creating
    // a verified plan
    let transaction =
      unsafe { session.context().get_or_create_active_transaction() };
    for stmt in statement.stmts.clone().into_iter() {
      transaction
        .create_verified_logical_plan(stmt.into())
        .await?;
    }

    client.portal_store().put_statement(Arc::new(stmt));
    client
      .send(PgWireBackendMessage::ParseComplete(ParseComplete::new()))
      .await?;

    Ok(())
  }

  /// Prepares the logical plan for the query and bind the parameters to it
  #[tracing::instrument(skip_all, level = "trace")]
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

    let statement_name = message
      .statement_name
      .as_deref()
      .unwrap_or(pgwire::api::DEFAULT_NAME);

    tracing::trace!(statement_name);
    if let Some(statement) = client.portal_store().get_statement(statement_name)
    {
      let query = statement.statement.clone();
      // If the query planning was successful, add the plan to the portal
      // state. It could fail if the placeholder type can't be resolved just
      // from the query itself and needs the paramter values as well
      let transaction =
        unsafe { session.context().get_or_create_active_transaction() };
      let state = match transaction
        .create_verified_logical_plan(query.stmts[0].clone())
        .await
      {
        Ok(plan) => {
          let (params, fields) = get_params_and_field_types(&plan)?;
          Some(
            ArenaPortalState::default()
              .set_query_plan(Some(plan))
              .set_params(Some(params))
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

  #[tracing::instrument(
    skip_all,
    err,
    fields(query_type = "extended"),
    level = "DEBUG"
  )]
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
    let stmts = &portal.statement.statement.stmts;
    let stmt = stmts[0].clone();
    let plan = portal.state.as_ref().and_then(|s| s.query_plan().clone());

    let params_values = portal
      .state
      .as_ref()
      .and_then(|portal_state| {
        portal_state.params().as_ref().map(|params| {
          params
            .iter()
            .zip(&portal.parameters)
            .enumerate()
            .map(|(index, (r#type, param))| {
              convert_bytes_to_scalar_value(
                index,
                param.as_ref(),
                r#type,
                &portal.parameter_format,
              )
            })
            .collect::<PgWireResult<Vec<ScalarValue>>>()
        })
      })
      .transpose()?;
    Self::execute_plan(&session, stmt, plan, params_values, FieldFormat::Binary)
      .await
  }

  // This is not needed since this handler has custom on_parse
  fn query_parser(&self) -> Arc<Self::QueryParser> {
    unimplemented!()
  }

  #[tracing::instrument(skip_all, level = "DEBUG")]
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
        portal.state.as_ref().and_then(|s| s.query_plan().clone()),
        portal.statement.as_ref(),
      ),
      StatementOrPortal::Statement(stmt) => (None, stmt),
    };

    let plan = match maybe_plan {
      Some(plan) => plan,
      None => {
        let session = self.get_client_session(client)?;
        let txn =
          unsafe { session.context().get_or_create_active_transaction() };
        let stmt = stmt.statement.stmts[0].clone();
        txn.create_verified_logical_plan(stmt).await?
      }
    };

    let (params, fields) = get_params_and_field_types(&plan)?;

    // logging params and fields here is fine since trace logs are stripped
    tracing::trace!("params = {:?}", params);
    tracing::trace!("fields = {:?}", fields);
    Ok(DescribeResponse::new(Some(params), fields))
  }

  #[tracing::instrument(skip_all, level = "trace")]
  async fn on_terminate<C>(&self, client: &mut C)
  where
    C: ClientInfo + Unpin + Send + Sync,
  {
    if let Ok(session) = self.get_client_session(client) {
      self.session_store.remove_session(session.id());
    }
  }
}

#[async_trait]
impl SimpleQueryHandler for ArenaSqlCluster {
  #[tracing::instrument(
    skip(self, client),
    fields(query_type = "simple"),
    level = "DEBUG"
  )]
  async fn do_query<'a, C>(
    &self,
    client: &mut C,
    query: &'a str,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo + Unpin + Send + Sync,
  {
    let parser = Arc::new(ArenaQueryParser {});
    let parsed_query = parser.parse_sql(query, &[Type::ANY]).await?;
    let session = match &parsed_query.client {
      AuthHeader::None => self.get_client_session(client),
      header => self.get_or_create_new_session(client, &header),
    }?;

    // It seems like, in Postgres, all the statements in a single query
    // are run in the same transaction unless BEING/COMMIT/ROLLBACK is
    // explicity used
    // If an error occurs when executing a query with more than one statement,
    // the further processing of the query should be stopped; meaning,
    // statements remaining after the statement that errored shouldn't be
    // executed
    let mut results = Vec::with_capacity(parsed_query.stmts.len());
    for stmt in parsed_query.stmts.into_iter() {
      let result =
        Self::execute_plan(&session, stmt, None, None, FieldFormat::Text)
          .await?;
      results.push(result);
    }
    Ok(results)
  }
}

fn get_params_and_field_types(
  plan: &LogicalPlan,
) -> PgWireResult<(Vec<Type>, Vec<FieldInfo>)> {
  // Expects placeholder to be in format "${index}"
  let params: Vec<Type> = plan
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
          .map(|t| {
            let arena_type =
              t.1.iter().find(|f| f.0 == "TYPE").map(|f| f.1.clone());
            datatype::derive_pg_type(&t.0, arena_type.as_ref())
          })
          .unwrap_or(Type::TEXT),
      )
    })
    .sorted_by(|a, b| Ord::cmp(&a.0, &b.0))
    .map(|(_, t)| t)
    .collect();

  // Note: Set params to None so that NoData response is sent when query
  // is of type Set
  let field = plan
    .schema()
    .fields()
    .iter()
    .filter_map(|f| {
      if f.name() == CTID_COLUMN {
        None
      } else {
        Some(datatype::to_field_info(
          f.field().as_ref(),
          FieldFormat::Text,
        ))
      }
    })
    .collect();

  Ok((params, field))
}

fn convert_bytes_to_scalar_value(
  index: usize,
  bytes: Option<&Bytes>,
  r#type: &Type,
  format: &Format,
) -> PgWireResult<ScalarValue> {
  let is_binary_format = match format {
    Format::UnifiedText => false,
    Format::UnifiedBinary => true,
    Format::Individual(format) => format[index] == 1,
  };
  let scalar = match *r#type {
    Type::BOOL => {
      ScalarValue::Boolean(bytes.map(|b| if b[0] > 0 { true } else { false }))
    }
    Type::INT4 => ScalarValue::Int32(
      bytes
        .map(|by| match is_binary_format {
          false => parse_from_text(index, by),
          true => by
            .as_bytes()
            .try_into()
            .map_err(|_| invalid_param_err(index))
            .map(|v| i32::from_be_bytes(v)),
        })
        .transpose()?,
    ),
    Type::INT8 => ScalarValue::Int64(
      bytes
        .map(|by| match is_binary_format {
          false => parse_from_text(index, by),
          true => by
            .as_bytes()
            .try_into()
            .map_err(|_| invalid_param_err(index))
            .map(|v| i64::from_be_bytes(v)),
        })
        .transpose()?,
    ),
    Type::TEXT | Type::VARCHAR => ScalarValue::Utf8(
      bytes.and_then(|b| std::str::from_utf8(&b).map(|s| s.to_owned()).ok()),
    ),
    Type::JSONB => {
      ScalarValue::Utf8(
        bytes
          .map(|b| {
            let raw_bytes = if is_binary_format {
              if b[0] != 1 {
                tracing::error!(
                  "Unsuported JSONB format; exepcted first byte to be 1"
                );
                return Err(Error::InternalError(format!(
                  "Unknown param format"
                )));
              }
              // start from index 1 since first byte is 1 for JSONB
              &b[1..]
            } else {
              b
            };
            std::str::from_utf8(raw_bytes)
              .map(|s| s.to_owned())
              .map_err(|e| {
                Error::InvalidDataType(format!("Invalid JSON: {:?}", e))
              })
          })
          .transpose()?,
      )
    }
    Type::FLOAT4_ARRAY => {
      return Ok(ScalarValue::List(Arc::new(
        ListArray::from_iter_primitive::<Float32Type, _, _>(
          bytes
            .map(|b| {
              let vector = Vec::<f32>::from_sql(&Type::FLOAT4_ARRAY, b)
                .map_err(|e| {
                  Error::InvalidParameter(format!(
                    "Invalid FLOAT4_ARRAY: {:?}",
                    e
                  ))
                })?;

              Ok::<_, Error>(vec![Some(vector.into_iter().map(|v| Some(v)))])
            })
            .transpose()?
            .unwrap_or_default(),
        ),
      )));
    }
    Type::TIMESTAMP => {
      return Ok(ScalarValue::Int64(
        bytes
          .map(|by| match is_binary_format {
            false => parse_from_text(index, by),
            true => by
              .as_bytes()
              .try_into()
              .map_err(|_| invalid_param_err(index))
              .map(|v| i64::from_be_bytes(v)),
          })
          .transpose()?,
      ))
    }
    _ => {
      unimplemented!("Converting bytes to ScalarValue for type {:?}", r#type)
    }
  };

  Ok(scalar)
}

fn parse_from_text<T: std::str::FromStr>(
  index: usize,
  bytes: &Bytes,
) -> Result<T, Error> {
  let str_value = std::str::from_utf8(bytes.as_bytes())
    .map_err(|_| invalid_param_err(index))?;
  str_value.parse::<T>().map_err(|_| invalid_param_err(index))
}

fn invalid_param_err(index: usize) -> Error {
  Error::InvalidParameter(format!("Invalid parameter at index {}", index))
}
