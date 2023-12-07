use sqlparser::ast::Statement as SQLStatement;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum QueryClient {
  Authenticated { id: String },
  New { user: String, database: String },
}

#[derive(Debug, Clone)]
pub struct ArenaQuery {
  pub client: Option<QueryClient>,
  pub stmts: Vec<Box<SQLStatement>>,
}

pub trait CommandString {
  fn command(&self) -> &'static str;
}

impl CommandString for Box<SQLStatement> {
  fn command(&self) -> &'static str {
    match self.as_ref() {
      SQLStatement::Query(_) => "SELECT",
      SQLStatement::Insert { .. } => "INSERT",
      SQLStatement::CreateDatabase { .. }
      | SQLStatement::CreateTable { .. } => "CREATE",
      SQLStatement::Delete { .. } => "DELETE",
      SQLStatement::AlterIndex { .. } => "ALTER",
      stmt => unimplemented!("Command not supported: {}", stmt),
    }
  }
}
