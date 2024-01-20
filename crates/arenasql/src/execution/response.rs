use std::pin::Pin;
use std::task::{Context, Poll};

use datafusion::arrow::array::as_primitive_array;
use datafusion::arrow::datatypes::Int64Type;
use datafusion::arrow::datatypes::UInt64Type;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::{DataFusionError, Result as DataFusionResult};
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use derivative::Derivative;
use futures::Stream;
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
  #[derivative(Debug = "ignore")]
  stream_completion_hook: Option<Pin<Box<StreamCompletionHook>>>,
}

impl ExecutionResponse {
  pub fn empty() -> Self {
    Self {
      record_batches: None,
      stream: None,
      stream_completion_hook: None,
    }
  }

  /// Datafusion doesn't execute some queries like INSERT until the stream
  /// is polled. So, poll the stream here for those types of query and return
  /// a new stream of the polled values
  pub async fn from_stream(
    stmt_type: &StatementType,
    stream: RecordBatchStream,
  ) -> ArenaResult<Self> {
    let (record_batches, stream) = match stmt_type {
      StatementType::Query | StatementType::Execute | StatementType::Set => {
        (None, Some(stream))
      }
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
      stream_completion_hook: None,
    })
  }

  pub fn set_stream_completion_hook(
    &mut self,
    hook: StreamCompletionHook,
  ) -> ArenaResult<()> {
    if self.stream_completion_hook.is_some() {
      panic!("Stream completion hook already set");
    } else if self.stream.is_some() {
      self.stream_completion_hook = Some(Box::pin(hook));
      Ok(())
    } else {
      // If the stream was already collected, call the hook
      let mut hook = hook.hook;
      let cb = hook.take().unwrap();
      cb()
    }
  }

  /// Panics if it's not a SELECT query; i.e. doesn't have a response
  /// stream
  pub fn get_stream(self) -> RecordBatchStream {
    let stream = self.stream.unwrap();
    if let Some(hook) = self.stream_completion_hook {
      Box::pin(RecordBatchStreamAdapter::new(
        stream.schema(),
        stream.chain(hook),
      ))
    } else {
      stream
    }
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
  pub fn get_modified_rows(&self) -> Option<usize> {
    self.record_batches.as_ref().map(|batch| {
      batch
        .iter()
        .flat_map(|b| {
          b.column_by_name("count")
            .map(|count| {
              as_primitive_array::<UInt64Type>(count)
                .iter()
                .map(|v| v.unwrap_or(0))
            })
            .unwrap()
        })
        .sum::<u64>() as usize
    })
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

pub struct StreamCompletionHook {
  hook: Option<Box<dyn (FnOnce() -> ArenaResult<()>) + Send + Sync>>,
}

unsafe impl Send for StreamCompletionHook {}
unsafe impl Sync for StreamCompletionHook {}

impl StreamCompletionHook {
  pub fn new<F>(hook: Box<F>) -> Self
  where
    F: (FnOnce() -> ArenaResult<()>) + Send + Sync + 'static,
  {
    Self { hook: Some(hook) }
  }
}

impl Stream for StreamCompletionHook {
  type Item = DataFusionResult<RecordBatch>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let hook = self.hook.take().unwrap();
    if let Err(err) = hook() {
      Poll::Ready(Some(Err(err.into())))
    } else {
      Poll::Ready(None)
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, None)
  }
}
