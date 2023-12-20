use sqlparser::ast::Statement as SQLStatement;

use crate::auth::AuthHeader;

#[derive(Debug, Clone)]
pub struct ArenaQuery {
  pub client: AuthHeader,
  pub stmts: Vec<Box<SQLStatement>>,
}
