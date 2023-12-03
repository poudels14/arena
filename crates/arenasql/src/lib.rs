mod df;
mod error;
pub(crate) mod utils;

pub mod parser;
pub mod runtime;
pub mod schema;
pub mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Error = error::Error;

pub use df::execution::{SessionConfig, SessionContext, Transaction};
pub use df::providers::{CatalogListProvider, SingleCatalogListProvider};

pub mod response {
  pub use crate::df::execution::response::*;
}

pub mod records {
  pub use crate::df::stream;
  pub use crate::df::{RecordBatch, RecordBatchStream};
  pub use datafusion::arrow::datatypes::{
    DataType as DatafusionDataType, Field as DatafusionField,
  };
}

pub mod arrow {
  pub use datafusion::arrow::array::{
    Array, ArrayAccessor, ArrayIter, ArrayRef, BinaryArray, BinaryBuilder,
    BooleanArray, BooleanBuilder, Float32Array, Float32Builder, Float64Array,
    Float64Builder, Int32Array, Int32Builder, Int64Array, Int64Builder,
    NullBuilder, StringArray, StringBuilder,
  };
}
