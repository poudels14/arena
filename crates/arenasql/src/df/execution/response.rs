use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::DataFusionError;
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use derivative::Derivative;
use futures::StreamExt;

use crate::records::RecordBatchStream;
use crate::Result as ArenaResult;

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
  /// Datafusion doesn't execute some queries like INSERT until the stream
  /// is polled. So, poll the stream here for those types of query and return
  /// a new stream of the polled values
  pub async fn create(
    plan: LogicalPlan,
    stream: RecordBatchStream,
  ) -> ArenaResult<Self> {
    match plan {
      LogicalPlan::Ddl(_) => Ok(Self {
        stmt_type: Type::Ddl,
        stream,
      }),
      LogicalPlan::Dml(_) => {
        let schema = stream.schema();
        let result: Vec<Result<RecordBatch, DataFusionError>> =
          stream.collect().await;

        Ok(Self {
          stmt_type: Type::Dml,
          stream: Box::pin(RecordBatchStreamAdapter::new(
            schema,
            futures::stream::iter(result),
          )),
        })
      }
      _ => Ok(Self {
        stmt_type: Type::Query,
        stream,
      }),
    }
  }
}
