use sqlparser::ast::Statement as SQLStatement;

use super::QueryClient;

#[derive(Debug, Clone)]
pub struct ArenaQuery {
  pub client: QueryClient,
  pub stmts: Vec<Box<SQLStatement>>,
}

pub trait SqlCommand {
  fn command(&self) -> &'static str;
  /// Whether the SQL is `SELECT` command
  fn is_query(&self) -> bool;
  fn is_begin(&self) -> bool;
  fn is_commit(&self) -> bool;
  fn is_rollback(&self) -> bool;
}

impl SqlCommand for &SQLStatement {
  #[inline]
  fn command(&self) -> &'static str {
    match self {
      SQLStatement::StartTransaction { .. } => "BEGIN",
      SQLStatement::Commit { .. } => "COMMIT",
      SQLStatement::Rollback { .. } => "ROLLBACK",
      SQLStatement::Query(_) => "SELECT",
      SQLStatement::Insert { .. } => "INSERT",
      SQLStatement::CreateDatabase { .. }
      | SQLStatement::CreateTable { .. } => "CREATE",
      SQLStatement::Delete { .. } => "DELETE",
      SQLStatement::Update { .. } => "UPDATE",
      SQLStatement::AlterIndex { .. } => "ALTER",
      stmt => unimplemented!("Command not supported: {}", stmt),
    }
  }

  #[inline]
  fn is_query(&self) -> bool {
    match self {
      SQLStatement::Query { .. } => true,
      _ => false,
    }
  }

  #[inline]
  fn is_begin(&self) -> bool {
    match self {
      SQLStatement::StartTransaction { .. } => true,
      _ => false,
    }
  }

  #[inline]
  fn is_commit(&self) -> bool {
    match self {
      SQLStatement::Commit { .. } => true,
      _ => false,
    }
  }

  #[inline]
  fn is_rollback(&self) -> bool {
    match self {
      SQLStatement::Rollback { .. } => true,
      _ => false,
    }
  }
}
