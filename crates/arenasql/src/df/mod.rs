use datafusion::arrow::record_batch;
use datafusion::physical_plan::SendableRecordBatchStream;

mod insert;
pub(crate) mod providers;
mod scan;

pub type RecordBatchStream = SendableRecordBatchStream;
pub type RecordBatch = record_batch::RecordBatch;

pub mod stream {
  pub use datafusion::physical_plan::RecordBatchStream;
}
