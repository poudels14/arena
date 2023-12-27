use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::hash::Hash;
use std::pin::Pin;
use std::sync::Arc;

use datafusion::arrow::datatypes::{Schema, SchemaRef};
use datafusion::common::{DFField, DFSchema, DFSchemaRef};
use datafusion::error::Result;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{Expr, LogicalPlan, UserDefinedLogicalNodeCore};
use datafusion::physical_expr::PhysicalSortExpr;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use datafusion::physical_plan::{
  DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning,
  SendableRecordBatchStream, Statistics,
};
use derivative::Derivative;
use derive_new::new;
use futures::{Stream, StreamExt};
use sqlparser::ast::Statement as SQLStatement;

use super::Transaction;
use crate::schema::DataFrame;

pub type ExecutionPlanExtension = Arc<
  dyn Fn(
      &Transaction,
      &SQLStatement,
    ) -> crate::Result<Option<Arc<dyn CustomExecutionPlan>>>
    + Send
    + Sync,
>;

pub trait CustomExecutionPlan: Send + Sync {
  fn schema(&self) -> SchemaRef;

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> crate::Result<ExecutionPlanResponse>;
}

pub type ExecutionPlanResponse =
  Pin<Box<dyn Stream<Item = crate::Result<DataFrame>> + Send>>;

#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct CustomExecutionPlanAdapter {
  #[derivative(Debug = "ignore")]
  inner: Arc<dyn CustomExecutionPlan>,
}

impl DisplayAs for CustomExecutionPlanAdapter {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut Formatter,
  ) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl ExecutionPlan for CustomExecutionPlanAdapter {
  #[inline]
  fn as_any(&self) -> &dyn Any {
    self
  }

  #[inline]
  fn schema(&self) -> SchemaRef {
    self.inner.schema()
  }

  #[inline]
  fn children(&self) -> Vec<Arc<dyn ExecutionPlan>> {
    vec![]
  }

  #[inline]
  fn with_new_children(
    self: Arc<Self>,
    _children: Vec<Arc<dyn ExecutionPlan>>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    unimplemented!()
  }

  #[inline]
  fn output_ordering(&self) -> Option<&[PhysicalSortExpr]> {
    None
  }

  #[inline]
  fn output_partitioning(&self) -> Partitioning {
    Partitioning::UnknownPartitioning(1)
  }

  #[inline]
  fn statistics(&self) -> Result<Statistics> {
    Ok(Statistics::new_unknown(&Schema::empty()))
  }

  #[inline]
  fn execute(
    &self,
    partition: usize,
    context: Arc<TaskContext>,
  ) -> Result<SendableRecordBatchStream> {
    let schema = self.schema();
    let df_stream = self.inner.execute(partition, context)?;
    Ok(Box::pin(RecordBatchStreamAdapter::new(
      self.schema(),
      df_stream
        .map(move |df| df.map(|df| df.to_record_batch(schema.clone()))?),
    )))
  }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, new)]
pub struct CustomLogicalPlan {
  schema: DFSchemaRef,
}

impl CustomLogicalPlan {
  pub fn create(schema: SchemaRef) -> Self {
    let schema = DFSchema::new_with_metadata(
      schema
        .fields
        .iter()
        .map(|f| {
          DFField::new_unqualified(
            f.name(),
            f.data_type().clone(),
            f.is_nullable(),
          )
        })
        .collect(),
      HashMap::new(),
    )
    .unwrap()
    .into();

    Self { schema }
  }
}

impl UserDefinedLogicalNodeCore for CustomLogicalPlan {
  fn name(&self) -> &str {
    "CustomLogicalPlan"
  }

  fn schema(&self) -> &DFSchemaRef {
    &self.schema
  }

  fn inputs(&self) -> Vec<&LogicalPlan> {
    vec![]
  }

  fn fmt_for_explain(&self, _f: &mut Formatter) -> std::fmt::Result {
    unimplemented!()
  }

  fn from_template(&self, _exprs: &[Expr], _inputs: &[LogicalPlan]) -> Self {
    self.clone()
  }

  fn expressions(&self) -> Vec<Expr> {
    vec![]
  }
}
