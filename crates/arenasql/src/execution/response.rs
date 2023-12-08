use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::DataFusionError;
use datafusion::logical_expr::LogicalPlan;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use derivative::Derivative;
use futures::StreamExt;

use crate::records::RecordBatchStream;
use crate::{Error, Result as ArenaResult};

#[derive(Derivative)]
#[derivative(Debug)]
pub enum Type {
  Unknown,
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
    stream: RecordBatchStream,
    plan: Option<LogicalPlan>,
  ) -> ArenaResult<Self> {
    match plan {
      Some(LogicalPlan::Ddl(_)) => Ok(Self {
        stmt_type: Type::Ddl,
        stream,
      }),
      Some(LogicalPlan::Dml(_)) | None => {
        let schema = stream.schema();
        let mut result: Vec<Result<RecordBatch, DataFusionError>> =
          stream.collect().await;

        if result.len() != 1 {
          return Err(Error::InternalError(format!(
            "Only one result expected from Dml query but got {}",
            result.len()
          )));
        }
        let batch = result.pop().unwrap()?;

        Ok(Self {
          stmt_type: Type::Dml,
          stream: Box::pin(RecordBatchStreamAdapter::new(
            schema,
            futures::stream::iter(vec![Ok(batch)]),
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
