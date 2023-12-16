use datafusion::arrow::array::as_primitive_array;
use datafusion::arrow::datatypes::Int64Type;
use datafusion::arrow::datatypes::UInt64Type;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::DataFusionError;
use datafusion::logical_expr::LogicalPlan;
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
  record_batches: Option<Vec<RecordBatch>>,
  #[derivative(Debug = "ignore")]
  stream: Option<RecordBatchStream>,
}

impl ExecutionResponse {
  /// Datafusion doesn't execute some queries like INSERT until the stream
  /// is polled. So, poll the stream here for those types of query and return
  /// a new stream of the polled values
  pub async fn from_stream(stream: RecordBatchStream) -> ArenaResult<Self> {
    Self::from_stream_and_plan(stream, None).await
  }

  pub async fn from_stream_and_plan(
    stream: RecordBatchStream,
    plan: Option<LogicalPlan>,
  ) -> ArenaResult<Self> {
    match plan {
      Some(LogicalPlan::Ddl(_)) => Ok(Self {
        stmt_type: Type::Ddl,
        record_batches: None,
        stream: Some(stream),
      }),
      Some(LogicalPlan::Dml(_)) | None => {
        let batches = stream
          .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
          .await;

        if batches.len() != 1 {
          return Err(Error::InternalError(format!(
            "Only one result expected from Dml query but got {}",
            batches.len()
          )));
        }

        let batches = batches
          .into_iter()
          .map(|b| Ok(b?))
          .collect::<ArenaResult<Vec<RecordBatch>>>()?;

        Ok(Self {
          stmt_type: Type::Dml,
          record_batches: Some(batches),
          stream: None,
        })
      }
      _ => Ok(Self {
        stmt_type: Type::Query,
        record_batches: None,
        stream: Some(stream),
      }),
    }
  }

  pub fn get_stream(self) -> ArenaResult<RecordBatchStream> {
    Ok(self.stream.unwrap())
  }

  /// Returns total number of rows
  /// Panics if called on queries that doesn't return rows
  pub async fn num_rows(self) -> ArenaResult<usize> {
    let batches = self
      .stream
      .unwrap()
      .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
      .await;

    let mut num_rows: usize = 0;
    for batch in batches.into_iter() {
      num_rows += batch?.num_rows();
    }

    Ok(num_rows)
  }

  /// Returns total number of modified rows
  /// Panics if called on non DML queries
  pub fn get_modified_rows(&self) -> usize {
    self
      .record_batches
      .as_ref()
      .unwrap()
      .iter()
      .flat_map(|b| {
        as_primitive_array::<UInt64Type>(b.column_by_name("count").unwrap())
          .iter()
          .map(|v| v.unwrap_or(0))
      })
      .sum::<u64>() as usize
  }

  /// This returns the single value of the record batch
  /// Panics if called on queries other than SELECT
  pub async fn get_count(self) -> ArenaResult<i64> {
    let mut batches = self
      .stream
      .unwrap()
      .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
      .await;

    if batches.len() != 1 {
      panic!("Expected a single record but got: {}", batches.len());
    };

    let batch = batches.pop().unwrap()?;
    if batch.num_columns() != 1 {
      panic!("Expected a single column but got: {}", batch.num_columns());
    }

    let arr = as_primitive_array::<Int64Type>(batch.column(0));
    Ok(arr.value(0))
  }
}
