#[cfg(test)]
mod tests;

mod df;
mod error;
pub(crate) mod execution;
pub(crate) mod utils;

pub mod parser;
pub mod runtime;
pub mod schema;
pub mod storage;
pub mod vectors;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Error = error::Error;

pub use df::providers::{CatalogListProvider, SingleCatalogListProvider};
pub use execution::{SessionConfig, SessionContext, Transaction};

pub mod common {
  pub use datafusion::common::{ScalarType, ScalarValue};
  pub use datafusion::logical_expr::LogicalPlan;
}

pub mod response {
  pub use crate::execution::response::*;
}

pub mod records {
  pub use crate::df::stream;
  pub use crate::df::{RecordBatch, RecordBatchStream};
  pub use datafusion::arrow::datatypes::{
    DataType as DatafusionDataType, Field as DatafusionField,
  };
}

// Re-exports
pub use bytes;
pub use postgres_types;

pub mod arrow {
  pub use datafusion::arrow::array::{
    Array, ArrayAccessor, ArrayIter, ArrayRef, BinaryArray, BinaryBuilder,
    BooleanArray, BooleanBuilder, Float32Array, Float32Builder, Float64Array,
    Float64Builder, Int32Array, Int32Builder, Int64Array, Int64Builder,
    ListArray, NullBuilder, StringArray, StringBuilder,
  };
}
