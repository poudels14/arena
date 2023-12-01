use async_trait::async_trait;
use pgwire::api::stmt::QueryParser;
use pgwire::error::{ErrorInfo, PgWireError, PgWireResult};

use super::ArenaQuery;

pub struct ArenaQueryParser;

#[async_trait]
impl QueryParser for ArenaQueryParser {
  type Statement = ArenaQuery;

  async fn parse_sql(
    &self,
    sql: &str,
    _types: &[pgwire::api::Type],
  ) -> PgWireResult<Self::Statement> {
    let stmts = arenasql::parser::parse(sql)
      .map_err(|e| {
        PgWireError::UserError(
          ErrorInfo::new("ERROR".to_owned(), "42601".to_owned(), e.to_string())
            .into(),
        )
      })?
      .into_iter()
      .map(|stmt| Box::new(stmt))
      .collect();

    Ok(ArenaQuery {
      client: None,
      stmts,
    })
  }
}
