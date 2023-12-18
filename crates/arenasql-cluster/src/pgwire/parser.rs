use async_trait::async_trait;
use pgwire::api::stmt::QueryParser;
use pgwire::error::PgWireResult;

use super::{ArenaQuery, QueryClient};

pub struct ArenaQueryParser;

#[async_trait]
impl QueryParser for ArenaQueryParser {
  type Statement = ArenaQuery;

  async fn parse_sql(
    &self,
    sql: &str,
    _types: &[pgwire::api::Type],
  ) -> PgWireResult<Self::Statement> {
    let stmts = arenasql::ast::parse_and_sanitize(sql)?
      .into_iter()
      .map(|stmt| Box::new(stmt))
      .collect();

    Ok(ArenaQuery {
      client: QueryClient::Unknown,
      stmts,
    })
  }
}
