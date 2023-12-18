use std::fmt::Debug;
use std::sync::Arc;

use arenasql::common::ScalarValue;
use async_trait::async_trait;
use futures::{Sink, SinkExt};
use pgwire::api::portal::Portal;
use pgwire::api::query::{
  ExtendedQueryHandler, SimpleQueryHandler, StatementOrPortal,
};
use pgwire::api::results::{DescribeResponse, Response};
use pgwire::api::stmt::QueryParser;
use pgwire::api::store::PortalStore;
use pgwire::api::{ClientInfo, Type};
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};
use pgwire::messages::extendedquery::{Bind, BindComplete};
use pgwire::messages::PgWireBackendMessage;

use super::portal::ArenaPortalState;
use super::statement::ArenaQuery;
use super::{ArenaPortalStore, ArenaQueryParser};
use crate::pgwire::datatype;
use crate::server::ArenaSqlCluster;

#[async_trait]
impl ExtendedQueryHandler for ArenaSqlCluster {
  type PortalState = ArenaPortalState;
  type PortalStore = ArenaPortalStore;
  type Statement = ArenaQuery;
  type QueryParser = ArenaQueryParser;

  /// Prepares the logical plan for the query and bind the parameters to it
  async fn on_bind<C>(&self, client: &mut C, message: Bind) -> PgWireResult<()>
  where
    C: ClientInfo + Sink<PgWireBackendMessage> + Unpin + Send + Sync,
    C::Error: Debug,
    PgWireError: From<<C as Sink<PgWireBackendMessage>>::Error>,
  {
    let session = self.get_client_session(client)?;
    let txn = session.create_transaction()?;

    let statement_name = message
      .statement_name()
      .as_deref()
      .unwrap_or(pgwire::api::DEFAULT_NAME);

    if let Some(statement) =
      self.portal_store(client).get_statement(statement_name)
    {
      let query = statement.statement();
      let plan = txn
        .create_verified_logical_plan(query.stmts[0].clone())
        .await?;
      drop(txn);

      let state = ArenaPortalState::default()
        .set_query_plan(Some(plan))
        .to_owned();

      let portal = Portal::try_new(&message, statement, Some(state))?;
      self.portal_store(client).put_portal(Arc::new(portal));
      client
        .send(PgWireBackendMessage::BindComplete(BindComplete::new()))
        .await?;
      Ok(())
    } else {
      Err(PgWireError::StatementNotFound(statement_name.to_owned()))
    }
  }

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

    let prams_vec: Vec<ScalarValue> = params
      .iter()
      .map(|param| {
        ScalarValue::Utf8(
          param
            .as_ref()
            .map(|p| std::str::from_utf8(p).unwrap().to_owned()),
        )
      })
      .collect();

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

    let final_plan = plan
      .replace_params_with_values(prams_vec.as_slice())
      .map_err(|e| arenasql::Error::DataFusionError(e.into()))?;

    let response = transaction.execute_logical_plan(final_plan).await?;
    // Commit the transaction if it's not a chained transaction
    // i.e. if it wasn't explicitly started by `BEGIN` command
    if !chained {
      transaction.commit()?;
    }
    Self::map_to_pgwire_response(&stmt, response).await
  }

  fn portal_store<C>(&self, client: &C) -> Arc<Self::PortalStore>
  where
    C: ClientInfo,
  {
    let client_id = client.socket_addr().to_string();
    match self.poral_stores.get(&client_id) {
      Some(store) => store.value().clone(),
      None => {
        let store = Arc::new(ArenaPortalStore::new());
        self.poral_stores.insert(client_id, store.clone());
        store
      }
    }
  }

  fn query_parser(&self) -> Arc<Self::QueryParser> {
    self.parser.clone()
  }

  async fn do_describe<C>(
    &self,
    _client: &mut C,
    target: StatementOrPortal<'_, Self::Statement, Self::PortalState>,
  ) -> PgWireResult<DescribeResponse>
  where
    C: ClientInfo + Send,
  {
    match target {
      StatementOrPortal::Portal(portal) => {
        let state = portal.state().as_ref().unwrap();

        return Ok(DescribeResponse::new(
          None,
          // TODO: if query plan doesn't exit, create a new one
          state
            .query_plan()
            .as_ref()
            .unwrap()
            .schema()
            .fields()
            .iter()
            .map(|f| datatype::to_field_info(f.field().as_ref()))
            .collect(),
        ));
      }
      _ => unimplemented!(),
    }
  }
}

#[async_trait]
impl SimpleQueryHandler for ArenaSqlCluster {
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
