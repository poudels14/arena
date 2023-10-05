pub mod errors;

use crate::query::Collection;
use crate::VectorDatabase;
use anyhow::{anyhow, bail, Result};
use errors::Error;
use sqlparser::ast::Statement::CreateTable;
use sqlparser::ast::{DataType, Ident};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

pub struct Client<'a> {
  db: &'a mut VectorDatabase,
}

impl<'a> Client<'a> {
  pub fn new(db: &'a mut VectorDatabase) -> Self {
    Self { db }
  }

  pub fn execute(&mut self, sql: &str) -> Result<()> {
    let dialect = GenericDialect {};
    let ast = Parser::parse_sql(&dialect, sql).unwrap();

    assert!(
      ast.len() == 1,
      "Unsupported SQL query [only 1 AST is expected but got {}]",
      ast.len()
    );

    match ast.get(0) {
      Some(CreateTable {
        or_replace: _,
        temporary: _,
        external: _,
        global: _,
        if_not_exists: _,
        transient: _,
        name,
        columns,
        constraints: _,
        hive_distribution: _,
        hive_formats: _,
        table_properties: _,
        with_options: _,
        file_format: _,
        location: _,
        query: _,
        without_rowid: _,
        like: _,
        clone: _,
        engine: _,
        default_charset: _,
        collation: _,
        on_commit: _,
        on_cluster: _,
        order_by: _,
        strict: _,
      }) => {
        assert!(name.0.len() == 1, "Only one table name expected");
        let name = name
          .0
          .get(0)
          .ok_or(anyhow!("Failed to parse name of the collection"))?
          .value
          .as_bytes();

        let dimension = columns
          .iter()
          .find(|c| c.name.value == "dimension")
          .ok_or(anyhow!(
            "Table must have dimension column of type `vector($vec_len)`"
          ))?;

        let dim = match &dimension.data_type {
          DataType::Custom(type_name, dim) => match type_name.0.get(0) {
            Some(ident) if *ident == Ident::from("vector") => {
              dim.get(0).unwrap().parse::<u16>()
            }
            _ => bail!(Error::InvalidDimensionColumn),
          },
          _ => bail!(Error::InvalidDimensionColumn),
        }?;

        self.db.create_collection(
          simdutf8::basic::from_utf8(name)?,
          Collection {
            dimension: dim,
            metadata: None,
          },
        )
      }
      _ => {
        bail!("Unsupported query");
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::sql::Client;
  use crate::VectorDatabase;
  use anyhow::Result;

  #[test]
  fn test() -> Result<()> {
    let db_path = "/tmp/testdb";

    let mut db = VectorDatabase::open(db_path, Default::default())?;
    let mut client = Client::new(&mut db);

    client.execute("CREATE TABLE uploads (dimension vector(384))")?;

    let cols = db.list_collections()?;
    assert_eq!(cols.len(), 1);
    assert_eq!(cols.get(0).unwrap().0, "uploads");
    println!("Collections = {:?}", cols);

    db.close()?;
    drop(db);
    VectorDatabase::destroy(db_path)?;

    Ok(())
  }
}
