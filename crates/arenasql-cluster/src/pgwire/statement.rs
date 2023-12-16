use sqlparser::ast::Statement as SQLStatement;

use super::QueryClient;

#[derive(Debug, Clone)]
pub struct ArenaQuery {
  pub client: QueryClient,
  pub stmts: Vec<Box<SQLStatement>>,
}
