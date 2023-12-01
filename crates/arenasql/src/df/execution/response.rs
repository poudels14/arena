use datafusion::logical_expr::LogicalPlan;
use derivative::Derivative;

use crate::records::RecordBatchStream;

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Type {
  Ddl,
  Dml,
  Query,
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ExecutionResponse {
  pub stmt_type: Type,
  #[derivative(Debug = "ignore")]
  pub stream: RecordBatchStream,
}

impl ExecutionResponse {
  pub fn from(plan: LogicalPlan, stream: RecordBatchStream) -> Self {
    match plan {
      LogicalPlan::Ddl(_) => Self {
        stmt_type: Type::Ddl,
        stream,
      },
      LogicalPlan::Dml(_) => Self {
        stmt_type: Type::Dml,
        stream,
      },
      _ => Self {
        stmt_type: Type::Query,
        stream,
      },
    }
  }
}
