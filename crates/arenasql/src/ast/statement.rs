use sqlparser::ast::Statement as SQLStatement;

pub trait StatementType {
  fn get_type(&self) -> &'static str;

  /// Whether this is a `INSERT` statement
  fn is_insert(&self) -> bool;

  /// Whether the SQL is `SELECT` statement
  fn is_query(&self) -> bool;
  fn is_begin(&self) -> bool;
  fn is_commit(&self) -> bool;
  fn is_rollback(&self) -> bool;
}

impl StatementType for &SQLStatement {
  #[inline]
  fn get_type(&self) -> &'static str {
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
      stmt => unimplemented!("Statement type not supported: {}", stmt),
    }
  }

  #[inline]
  fn is_insert(&self) -> bool {
    match self {
      SQLStatement::Insert { .. } => true,
      _ => false,
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
