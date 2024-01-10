use std::any::Any;
use std::collections::HashMap;
use std::fmt::Formatter;
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
    _exprs: Vec<Expr>,
    _inputs: Vec<LogicalPlan>,
  ) -> crate::Result<ExecutionPlanResponse>;

  /// Returns all expressions in the logical plan node for this plan
  fn list_expressions(&self) -> Vec<Expr> {
    vec![]
  }
}

pub type ExecutionPlanResponse =
  Pin<Box<dyn Stream<Item = crate::Result<DataFrame>> + Send>>;

#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct CustomExecutionPlanAdapter {
  #[derivative(Debug = "ignore")]
  inner: Arc<dyn CustomExecutionPlan>,
  exprs: Vec<Expr>,
  inputs: Vec<LogicalPlan>,
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
    let df_stream = self.inner.execute(
      partition,
      context,
      self.exprs.clone(),
      self.inputs.clone(),
    )?;
    Ok(Box::pin(RecordBatchStreamAdapter::new(
      self.schema(),
      df_stream
        .map(move |df| df.map(|df| df.to_record_batch(schema.clone()))?),
    )))
  }
}

#[derive(Derivative)]
#[derivative(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CustomPlanAdapter {
  #[derivative(PartialEq = "ignore", Hash = "ignore", Debug = "ignore")]
  pub(crate) inner: Arc<dyn CustomExecutionPlan>,
  schema: DFSchemaRef,
  exprs: Vec<Expr>,
  inputs: Vec<LogicalPlan>,
}

impl CustomPlanAdapter {
  pub fn create(plan: Arc<dyn CustomExecutionPlan>) -> Self {
    let schema = DFSchema::new_with_metadata(
      plan
        .schema()
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

    Self {
      inner: plan,
      schema,
      exprs: vec![],
      inputs: vec![],
    }
  }

  pub fn get_execution_plan(&self) -> CustomExecutionPlanAdapter {
    CustomExecutionPlanAdapter {
      inner: self.inner.clone(),
      exprs: self.exprs.clone(),
      inputs: self.inputs.clone(),
    }
  }
}

impl DisplayAs for CustomPlanAdapter {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut Formatter,
  ) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl UserDefinedLogicalNodeCore for CustomPlanAdapter {
  fn name(&self) -> &str {
    "CustomPlanAdapter"
  }

  fn schema(&self) -> &DFSchemaRef {
    &self.schema
  }

  fn inputs(&self) -> Vec<&LogicalPlan> {
    vec![]
  }

  fn fmt_for_explain(&self, f: &mut Formatter) -> std::fmt::Result {
    f.write_str("CustomPlanAdapter")
  }

  fn from_template(&self, exprs: &[Expr], inputs: &[LogicalPlan]) -> Self {
    let mut clone = self.clone();
    clone.exprs = exprs.to_vec();
    clone.inputs = inputs.to_vec();
    clone
  }

  fn expressions(&self) -> Vec<Expr> {
    self.inner.list_expressions()
  }
}
