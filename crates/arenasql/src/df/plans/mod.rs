pub(crate) mod create_index;
pub(crate) mod delete_rows;
pub(crate) mod insert_rows;
pub(crate) mod scan_table;
pub(crate) mod update_rows;

use std::sync::Arc;

use datafusion::catalog::schema::SchemaProvider as DfSchemaProvider;
use datafusion::execution::context::SessionState;
use datafusion::physical_plan::ExecutionPlan;
use datafusion::sql::ResolvedTableReference;
use datafusion::sql::TableReference;
use sqlparser::ast::Statement as SQLStatement;

use self::create_index::{CreateIndex, CreateIndexExecutionPlanBuilder};
use crate::storage::Transaction;
use crate::{bail, Error, Result};

macro_rules! bail_unsupported_query {
  ($msg:literal) => {
    return Err(Error::UnsupportedQuery(format!($msg)));
  };
}

/// Returns a custom execution plan if the query execution is
/// supported using custom plan. This is needed to support queries
/// like `CREATE INDEX ...` that is not supported by datafusion
pub async fn get_custom_execution_plan(
  state: &SessionState,
  transaction: &Transaction,
  stmt: &Box<SQLStatement>,
) -> Result<Option<Arc<dyn ExecutionPlan>>> {
  match stmt.as_ref() {
    SQLStatement::CreateIndex {
      name,
      table_name,
      columns,
      unique,
      if_not_exists,
      // Features below this are not supported
      concurrently,
      using,
      nulls_distinct,
      predicate,
      include,
    } => {
      if *concurrently {
        bail_unsupported_query!("`CONCURRENTLY` is not supported yet");
      } else if using.is_some() {
        bail_unsupported_query!("`USING` is not supported yet");
      } else if nulls_distinct.is_some() {
        bail_unsupported_query!("`NULLS NOT DISTINCT` is not supported yet");
      } else if predicate.is_some() {
        bail_unsupported_query!("Partial index is not supported yet");
      } else if !include.is_empty() {
        bail_unsupported_query!("`INCLUDE` is not supported yet");
      }

      let table_name = table_name.to_string();
      let table_ref = get_table_ref(state, &table_name);
      let table_name = table_ref.table.as_ref().to_owned();

      let schema_provider = get_schema_provider(state, table_ref)?;

      if !schema_provider.table_exist(&table_name) {
        bail!(Error::RelationDoesntExist(table_name));
      }

      let table = transaction.state().get_table(&table_name).unwrap();
      let column_projection = columns
        .to_vec()
        .iter()
        .map(|c| c.to_string())
        .map(|col_name| {
          table
            .columns
            .iter()
            .position(|c| c.name == col_name)
            .ok_or_else(|| Error::ColumnDoesntExist(col_name.to_owned()))
        })
        .collect::<Result<Vec<usize>>>()?;

      let create_index = CreateIndex {
        name: name.as_ref().map(|n| n.to_string()),
        table,
        columns: column_projection,
        unique: *unique,
        if_not_exists: *if_not_exists,
      };

      return Ok(Some(Arc::new(
        CreateIndexExecutionPlanBuilder::default()
          .transaction(transaction.clone())
          .create_index(create_index)
          .build()
          .unwrap(),
      )));
    }
    _ => {}
  }
  Ok(None)
}

pub fn get_table_ref<'a>(
  state: &'a SessionState,
  table_name: &'a str,
) -> ResolvedTableReference<'a> {
  let table_ref = TableReference::parse_str(&table_name).to_owned();
  let catalog = &state.config_options().catalog;
  table_ref
    .clone()
    .resolve(&catalog.default_catalog, &catalog.default_schema)
}

/// Returns error if schema isn't found for the given table
pub fn get_schema_provider(
  state: &SessionState,
  table_ref: ResolvedTableReference<'_>,
) -> Result<Arc<dyn DfSchemaProvider>> {
  state
    .catalog_list()
    .catalog(&table_ref.catalog)
    // Catalog must exist!
    .unwrap()
    .schema(&table_ref.schema)
    .ok_or_else(|| {
      Error::SchemaDoesntExist(table_ref.schema.as_ref().to_owned())
    })
}
