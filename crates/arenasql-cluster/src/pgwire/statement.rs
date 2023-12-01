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
      SQLStatement::Insert {
        or: _,
        into: _,
        table_name: _,
        columns: _,
        overwrite: _,
        source: _,
        partitioned: _,
        after_columns: _,
        table: _,
        on: _,
        returning: _,
      } => "INSERT",
      SQLStatement::CreateDatabase {
        db_name: _,
        if_not_exists: _,
        location: _,
        managed_location: _,
      } => "CREATE",
      SQLStatement::Delete {
        tables: _,
        from: _,
        using: _,
        selection: _,
        returning: _,
      } => "DELETE",
      SQLStatement::AlterIndex {
        name: _,
        operation: _,
      } => "ALTER",
      _ => unimplemented!(),
    }
  }
}
