use std::sync::Arc;

use async_trait::async_trait;
use pgwire::api::portal::Portal;
use pgwire::api::query::{
  ExtendedQueryHandler, SimpleQueryHandler, StatementOrPortal,
};
use pgwire::api::results::{DescribeResponse, Response};
use pgwire::api::stmt::QueryParser;
use pgwire::api::{ClientInfo, Type};
use pgwire::error::{ErrorInfo, PgWireResult};
use pgwire::messages::extendedquery::Bind;

use super::statement::ArenaQuery;
use super::{ArenaPortalStore, ArenaQueryParser};
use crate::server::ArenaSqlCluster;

#[async_trait]
impl ExtendedQueryHandler for ArenaSqlCluster {
  type PortalStore = ArenaPortalStore;
  type Statement = ArenaQuery;
  type QueryParser = ArenaQueryParser;

  async fn on_bind<C>(&self, _client: &mut C, _bind: Bind) -> PgWireResult<()>
  where
    C: ClientInfo + Send,
  {
    // self.poral_store.put_portal(Arc::new(
    //   Portal::try_new(
    //     &bind,
    //     Arc::new(StoredStatement::new(
    //       "stored_stmt_1".to_owned(),
    //       ArenaQuery {
    //         client: None,
    //         stmts: vec![],
    //       },
    //       vec![],
    //     )),
    //   )
    //   .unwrap(),
    // ));
    unimplemented!();
  }

  async fn do_query<'p, 'h: 'p, C>(
    &'h self,
    _client: &mut C,
    _portal: &'p Portal<ArenaQuery>,
    _max_rows: usize,
  ) -> PgWireResult<Response<'p>>
  where
    C: ClientInfo + Send,
  {
    unimplemented!()
  }

  fn portal_store(&self) -> Arc<Self::PortalStore> {
    self.poral_store.clone()
  }

  fn query_parser(&self) -> Arc<Self::QueryParser> {
    self.parser.clone()
  }

  async fn do_describe<C>(
    &self,
    _client: &mut C,
    _target: StatementOrPortal<'_, Self::Statement>,
  ) -> PgWireResult<DescribeResponse> {
    unimplemented!()
  }
}

#[async_trait]
impl SimpleQueryHandler for ArenaSqlCluster {
  async fn do_query<'a, C>(
    &self,
    client: &C,
    query: &'a str,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo + Unpin + Send + Sync,
  {
    let parsed_query = self.parser.parse_sql(query, &[Type::ANY]).await?;
    let results_fut = self.execute_query(client, &parsed_query);

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
