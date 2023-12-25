use sqlparser::ast::Statement as SQLStatement;

#[derive(Debug, PartialEq)]
pub enum StatementType {
  Begin,
  Commit,
  Rollback,
  Query,
  Insert,
  Create,
  Drop,
  Delete,
  Update,
  Alter,
  Execute,
}

impl From<&SQLStatement> for StatementType {
  fn from(stmt: &SQLStatement) -> Self {
    match stmt {
      SQLStatement::StartTransaction { .. } => Self::Begin,
      SQLStatement::Commit { .. } => Self::Commit,
      SQLStatement::Rollback { .. } => Self::Rollback,
      SQLStatement::Query(_) => Self::Query,
      SQLStatement::Insert { .. } => Self::Insert,
      SQLStatement::CreateDatabase { .. }
      | SQLStatement::CreateTable { .. }
      | SQLStatement::CreateIndex { .. } => Self::Create,
      SQLStatement::Delete { .. } => Self::Delete,
      SQLStatement::Update { .. } => Self::Update,
      SQLStatement::AlterIndex { .. } => Self::Alter,
      SQLStatement::Execute { .. } => Self::Execute,
      SQLStatement::Drop { .. } => Self::Drop,
      stmt => unimplemented!("Statement type not supported: {}", stmt),
    }
  }
}

impl StatementType {
  #[inline]
  pub fn to_string(&self) -> &'static str {
    match self {
      Self::Begin => "BEGIN",
      Self::Commit => "COMMIT",
      Self::Rollback => "ROLLBACK",
      Self::Query => "SELECT",
      Self::Insert => "INSERT",
      Self::Create => "CREATE",
      Self::Drop => "DROP",
      Self::Delete => "DELETE",
      Self::Update => "UPDATE",
      Self::Alter => "ALTER",
      Self::Execute => "EXECUTE",
    }
  }

  #[inline]
  pub fn is_insert(&self) -> bool {
    *self == Self::Insert
  }

  #[inline]
  pub fn is_query(&self) -> bool {
    *self == Self::Query
  }

  #[inline]
  pub fn is_begin(&self) -> bool {
    *self == Self::Begin
  }

  #[inline]
  pub fn is_commit(&self) -> bool {
    *self == Self::Commit
  }

  #[inline]
  pub fn is_rollback(&self) -> bool {
    *self == Self::Rollback
  }
}
