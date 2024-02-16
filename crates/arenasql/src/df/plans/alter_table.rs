use std::fmt;
use std::sync::Arc;

use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{Expr, LogicalPlan};
use datafusion::physical_plan::{DisplayAs, DisplayFormatType};
use derivative::Derivative;
use derive_builder::Builder;
use futures::StreamExt;
use sqlparser::ast::{
  AlterTableOperation, ColumnDef, Statement as SQLStatement,
};

use crate::df::providers::{get_schema_provider, get_table_ref};
use crate::error::Error;
use crate::execution::{CustomExecutionPlan, Transaction};
use crate::execution::{ExecutionPlanResponse, TransactionHandle};
use crate::schema::{Column, ColumnProperty, DataFrame, DataType, Table};
use crate::{bail, Result};

/// Returns a custom execution plan extension to create index
#[tracing::instrument(skip_all, fields(name = "alter_table"), level = "trace")]
pub fn extension(
  transaction: &Transaction,
  stmt: &SQLStatement,
) -> Result<Option<Arc<dyn CustomExecutionPlan>>> {
  match stmt {
    SQLStatement::AlterTable {
      name,
      if_exists,
      only: _,
      operations,
    } => {
      if *if_exists {
        bail!(Error::UnsupportedQuery(format!(
          "`IF EXISTTS` is not supported yet"
        )));
      }

      let state = transaction.datafusion_context().state();
      let table_name = name.to_string();
      let table_ref = get_table_ref(&state, &table_name);
      let table_name = table_ref.table.as_ref().to_owned();

      let schema_provider = get_schema_provider(&state, &table_ref)?;
      if !schema_provider.table_exist(&table_name) {
        bail!(Error::RelationDoesntExist(table_name));
      }

      let columns = operations
        .iter()
        .map(|op| {
          if let AlterTableOperation::AddColumn { column_def, .. } = op {
            Ok(column_def.to_owned())
          } else {
            Err(Error::UnsupportedOperation(format!("{:?}", operations)))
          }
        })
        .collect::<Result<Vec<ColumnDef>>>()?;

      let table = transaction
        .handle()
        .get_table(&table_ref.schema, &table_name)
        .unwrap();
      return Ok(Some(Arc::new(
        AddColumnExecutionPlanBuilder::default()
          .transaction(transaction.handle().clone())
          .add_column(AddColumn {
            catalog: table_ref.catalog.as_ref().into(),
            schema: table_ref.schema.as_ref().into(),
            table,
            columns: columns.into(), // operations: operations.to_owned().into(),
          })
          .build()
          .unwrap(),
      )));
    }
    _ => {}
  }
  Ok(None)
}

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct AddColumnExecutionPlan {
  #[derivative(Debug = "ignore")]
  transaction: TransactionHandle,
  add_column: AddColumn,
}

#[derive(Debug, Clone)]
pub struct AddColumn {
  catalog: Arc<str>,
  schema: Arc<str>,
  table: Arc<Table>,
  columns: Arc<Vec<ColumnDef>>,
}

impl DisplayAs for AddColumnExecutionPlan {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut fmt::Formatter,
  ) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl CustomExecutionPlan for AddColumnExecutionPlan {
  fn schema(&self) -> SchemaRef {
    Arc::new(Schema::empty())
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
    _exprs: Vec<Expr>,
    _inputs: Vec<LogicalPlan>,
  ) -> crate::Result<ExecutionPlanResponse> {
    let add_column = self.add_column.clone();
    let transaction = self.transaction.clone();
    let stream = futures::stream::once(async move {
      let AddColumn {
        catalog,
        schema,
        table,
        columns,
      } = add_column;

      let mut table = table.as_ref().clone();
      let table_lock = transaction
        .acquire_table_schema_write_lock(schema.as_ref(), &table.name)
        .await?;

      let storage_handler = transaction.lock(true)?;
      columns
        .iter()
        .map(|col| {
          let new_column = Column {
            id: table.columns.len() as u8,
            name: col.name.value.clone(),
            data_type: DataType::from_column_def(&col, None)?,
            properties: ColumnProperty::DEFAULT,
            default_value: None,
          };
          table.columns.push(new_column);
          Ok(())
        })
        .collect::<Result<()>>()?;
      storage_handler.put_table_schema(&catalog, &schema, &table)?;
      transaction.hold_table_schema_lock(Arc::new(table), table_lock)?;
      Ok(DataFrame::empty())
    })
    .boxed();

    Ok(Box::pin(stream))
  }
}
