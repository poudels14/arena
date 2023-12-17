#[cfg(test)]
mod tests;

mod df;
mod error;
pub(crate) mod execution;
pub(crate) mod utils;

pub mod ast;
pub mod runtime;
pub mod schema;
pub mod storage;
pub mod vectors;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Error = error::Error;

pub use datafusion::catalog::{
  CatalogList as DatafusionCatalogList,
  CatalogProvider as DatafusionCatalogProvider,
};
pub use df::providers::{
  CatalogListProvider, CatalogProvider, SchemaProviderBuilder,
  SingleCatalogListProvider,
};
pub use execution::{
  SessionConfig, SessionContext, Transaction, DEFAULT_SCHEMA_NAME,
};

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
    DataType as DatafusionDataType, Field as DatafusionField, SchemaRef,
  };
}

// Re-exports
pub use bytes;
pub use postgres_types;

pub mod arrow {
  pub use datafusion::arrow::array::{
    as_boolean_array, as_generic_list_array, as_primitive_array,
    as_string_array, Array, ArrayAccessor, ArrayIter, ArrayRef, BinaryArray,
    BinaryBuilder, BooleanArray, BooleanBuilder, Float32Array, Float32Builder,
    Float64Array, Float64Builder, Int16Array, Int32Array, Int32Builder,
    Int64Array, Int64Builder, ListArray, NullBuilder, StringArray,
    StringBuilder, UInt16Array, UInt32Array, UInt64Array,
  };
  pub use datafusion::arrow::datatypes::{
    Float32Type, Float64Type, Int32Type, UInt32Type, UInt64Type,
  };
}
