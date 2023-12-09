use std::any::Any;
use std::fmt;
use std::sync::Arc;

use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::common::Statistics;
use datafusion::error::Result;
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

use crate::schema::{IndexType, Table};
use crate::storage::Transaction;
use crate::{bail, df_error, Error};

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct CreateIndexExecutionPlan {
  #[derivative(Debug = "ignore")]
  transaction: Transaction,
  create_index: CreateIndex,
  stmt: Box<SQLStatement>,
}

#[derive(Debug, Clone)]
pub struct CreateIndex {
  /// Index name
  pub name: Option<String>,
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
    unimplemented!()
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
      let mut table_lock =
        state.acquire_table_schema_write_lock(&table.name).await?;

      let storage_handler = transaction.lock()?;
      let index_id = storage_handler.get_next_table_index_id()?;
      table.add_index(index_id, index_type, index_name)?;

      storage_handler.put_table_schema(
        &state.catalog(),
        &state.schema(),
        &table,
      )?;

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
