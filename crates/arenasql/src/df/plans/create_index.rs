use std::any::Any;
use std::fmt;
use std::sync::Arc;

use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::common::Statistics;
use datafusion::error::Result;
use datafusion::execution::context::SessionState;
use datafusion::execution::TaskContext;
use datafusion::physical_expr::PhysicalSortExpr;
use datafusion::physical_plan::metrics::MetricsSet;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use datafusion::physical_plan::{
  DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning,
  SendableRecordBatchStream,
};
use derivative::Derivative;
use derive_builder::Builder;
use futures::StreamExt;
use sqlparser::ast::Statement as SQLStatement;

use crate::df::providers::{get_schema_provider, get_table_ref};
use crate::schema::{IndexType, Row, Table, TableIndex};
use crate::storage::{KeyValueGroup, StorageHandler, Transaction};
use crate::{bail, df_error, table_rows_prefix_key, Error};

macro_rules! bail_unsupported_query {
  ($msg:literal) => {
    return Err(Error::UnsupportedQuery(format!($msg)));
  };
}

/// Returns a custom execution plan extension to create index
pub fn extension(
  state: &SessionState,
  transaction: &Transaction,
  stmt: &Box<SQLStatement>,
) -> crate::Result<Option<Arc<dyn ExecutionPlan>>> {
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

      let schema_provider = get_schema_provider(state, &table_ref)?;

      if !schema_provider.table_exist(&table_name) {
        bail!(Error::RelationDoesntExist(table_name));
      }

      let table = transaction
        .state()
        .get_table(&table_ref.schema, &table_name)
        .unwrap();
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
        .collect::<crate::Result<Vec<usize>>>()?;

      let create_index = CreateIndex {
        name: name.as_ref().map(|n| n.to_string()),
        catalog: table_ref.catalog.as_ref().into(),
        schema: table_ref.schema.as_ref().into(),
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

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct CreateIndexExecutionPlan {
  #[derivative(Debug = "ignore")]
  transaction: Transaction,
  create_index: CreateIndex,
}

#[derive(Debug, Clone)]
pub struct CreateIndex {
  /// Index name
  pub name: Option<String>,
  pub catalog: Arc<str>,
  pub schema: Arc<str>,
  pub table: Arc<Table>,
  /// Column projection on the table
  pub columns: Vec<usize>,
  pub unique: bool,
  pub if_not_exists: bool,
}

impl DisplayAs for CreateIndexExecutionPlan {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut fmt::Formatter,
  ) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl ExecutionPlan for CreateIndexExecutionPlan {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<SendableRecordBatchStream> {
    let transaction = self.transaction.clone();
    let create_index = self.create_index.clone();
    let stream = futures::stream::once(async move {
      let CreateIndex {
        name: index_name,
        catalog,
        schema,
        table,
        columns,
        unique,
        if_not_exists,
      } = create_index;

      let mut table = table.as_ref().clone();
      let index_with_same_name_exist = index_name
        .as_ref()
        .map(|n| table.indexes.iter().any(|idx| idx.name == *n))
        .unwrap_or(false);

      if index_with_same_name_exist {
        if if_not_exists {
          return Ok(RecordBatch::new_empty(Arc::new(Schema::empty())));
        } else {
          bail!(df_error!(Error::RelationAlreadyExists(index_name.unwrap())));
        }
      }

      let index_type = match unique {
        true => IndexType::Unique(columns),
        false => IndexType::NonUnique(columns),
      };

      let state = transaction.state();
      let mut table_lock = state
        .acquire_table_schema_write_lock(schema.as_ref(), &table.name)
        .await?;

      let storage_handler = transaction.lock(true)?;
      let index_id = storage_handler.get_next_table_index_id()?;
      let new_index = table.add_index(index_id, index_type, index_name)?;

      storage_handler.put_table_schema(&catalog, &schema, &table)?;

      backfill_index_data(&storage_handler, &table, &new_index)?;

      table_lock.table = Some(Arc::new(table));
      state.hold_table_schema_lock(table_lock)?;

      Ok(RecordBatch::new_empty(Arc::new(Schema::empty())))
    })
    .boxed();

    Ok(Box::pin(RecordBatchStreamAdapter::new(
      Arc::new(Schema::empty()),
      stream,
    )))
  }

  fn schema(&self) -> SchemaRef {
    unimplemented!()
  }

  fn with_new_children(
    self: Arc<Self>,
    _children: Vec<Arc<dyn ExecutionPlan>>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    unimplemented!()
  }

  fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
    unimplemented!()
  }

  fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
    None
  }

  fn output_partitioning(&self) -> Partitioning {
    Partitioning::UnknownPartitioning(1)
  }

  fn metrics(&self) -> Option<MetricsSet> {
    None
  }

  fn statistics(&self) -> Result<Statistics> {
    Ok(Statistics::new_unknown(&Schema::empty()))
  }
}

fn backfill_index_data(
  storage_handler: &StorageHandler,
  table: &Table,
  new_index: &TableIndex,
) -> Result<()> {
  let mut rows_iter = storage_handler
    .kv
    .scan_with_prefix(KeyValueGroup::Rows, &table_rows_prefix_key!(table.id))?;

  let table_row_prefix = table_rows_prefix_key!(table.id);
  while let Some((row_key, row_bytes)) = rows_iter.get() {
    let row_id_bytes = &row_key[table_row_prefix.len()..];
    let row = storage_handler
      .serializer
      .deserialize::<Row<'_>>(row_bytes)?;

    storage_handler.add_row_to_index(table, &new_index, row_id_bytes, &row)?;
    rows_iter.next();
  }
  Ok(())
}
