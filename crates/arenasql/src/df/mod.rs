use datafusion::arrow::record_batch;
use datafusion::physical_plan::SendableRecordBatchStream;

pub(crate) mod plans;
pub(crate) mod providers;

pub type RecordBatchStream = SendableRecordBatchStream;
pub type RecordBatch = record_batch::RecordBatch;

pub mod stream {
  pub use datafusion::physical_plan::RecordBatchStream;
}
