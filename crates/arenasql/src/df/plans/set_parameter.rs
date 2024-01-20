use std::pin::Pin;
use std::sync::Arc;

use datafusion::arrow::array::UInt64Array;
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{Expr, LogicalPlan};
use futures::{FutureExt, Stream};
use sqlparser::ast::Statement as SQLStatement;

use crate::execution::{CustomExecutionPlan, Transaction};
use crate::schema::DataFrame;
use crate::Result;

#[tracing::instrument(
  skip_all,
  fields(name = "set_parameter"),
  level = "trace"
)]
pub fn extension(
  _transaction: &Transaction,
  stmt: &SQLStatement,
) -> Result<Option<Arc<dyn CustomExecutionPlan>>> {
  match stmt {
    SQLStatement::SetTimeZone { .. } | SQLStatement::SetVariable { .. } => {
      Ok(Some(Arc::new(SetParameterExecution {})))
    }
    _ => Ok(None),
  }
}

#[derive(Clone)]
pub struct SetParameterExecution {}

impl CustomExecutionPlan for SetParameterExecution {
  fn schema(&self) -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
      "count",
      DataType::UInt64,
      false,
    )]))
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
    _exprs: Vec<Expr>,
    _inputs: Vec<LogicalPlan>,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<DataFrame>> + Send>>> {
    let fut = async move {
      Ok(DataFrame::from_arrays(vec![Arc::new(UInt64Array::from(
        vec![0 as u64],
      ))]))
    }
    .boxed();
    Ok(Box::pin(futures::stream::once(fut)))
  }
}
