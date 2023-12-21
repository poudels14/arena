use datafusion::arrow::array::as_primitive_array;
use datafusion::arrow::datatypes::Int64Type;
use datafusion::arrow::datatypes::UInt64Type;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::DataFusionError;
use derivative::Derivative;
use futures::StreamExt;

use crate::ast::statement::StatementType;
use crate::datafusion::RecordBatchStream;
use crate::Result as ArenaResult;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct ExecutionResponse {
  #[derivative(Debug = "ignore")]
  record_batches: Option<Vec<RecordBatch>>,
  #[derivative(Debug = "ignore")]
  stream: Option<RecordBatchStream>,
}

impl ExecutionResponse {
  /// Datafusion doesn't execute some queries like INSERT until the stream
  /// is polled. So, poll the stream here for those types of query and return
  /// a new stream of the polled values
  pub async fn from_stream(
    stmt_type: &StatementType,
    stream: RecordBatchStream,
  ) -> ArenaResult<Self> {
    let (record_batches, stream) = match stmt_type {
      StatementType::Query | StatementType::Execute => (None, Some(stream)),
      _ => {
        let batches = stream
          .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
          .await;

        let batches = batches
          .into_iter()
          .map(|b| Ok(b?))
          .collect::<ArenaResult<Vec<RecordBatch>>>()?;
        (Some(batches), None)
      }
    };

    Ok(Self {
      record_batches,
      stream,
    })
  }

  /// Panics if it's not a SELECT query
  pub fn get_stream(self) -> RecordBatchStream {
    self.stream.unwrap()
  }

  pub async fn collect_batches(self) -> ArenaResult<Vec<RecordBatch>> {
    let batches = self
      .stream
      .unwrap()
      .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
      .await;

    batches
      .into_iter()
      .map(|b| Ok(b?))
      .collect::<ArenaResult<Vec<RecordBatch>>>()
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
    let batches = self
      .stream
      .unwrap()
      .collect::<Vec<Result<RecordBatch, DataFusionError>>>()
      .await;

    batches
      .into_iter()
      .map(|batch| {
        let batch = batch?;
        if batch.num_columns() != 1 {
          panic!("Expected a single column but got: {}", batch.num_columns());
        }
        let arr = as_primitive_array::<Int64Type>(batch.column(0));
        Ok(arr.value(0))
      })
      .sum()
  }
}
