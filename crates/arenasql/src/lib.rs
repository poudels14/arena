#[cfg(test)]
mod tests;

mod df;
mod error;
pub(crate) mod utils;

pub mod ast;
pub mod execution;
pub mod runtime;
pub mod schema;
pub mod storage;
pub mod vectors;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Error = error::Error;

pub use df::providers::{
  CatalogListProvider, CatalogProvider, SchemaProviderBuilder,
  SingleCatalogListProvider,
};

pub mod rocks {
  pub use rocksdb::backup::{
    BackupEngine, BackupEngineInfo, BackupEngineOptions, RestoreOptions,
  };
  pub use rocksdb::checkpoint::Checkpoint;
  pub use rocksdb::Env;
  pub use rocksdb::Error;
}

pub use chrono;
pub use pgwire;

pub mod datafusion {
  pub use datafusion::arrow::datatypes::{
    DataType as DatafusionDataType, Field as DatafusionField, Fields, Schema,
    SchemaRef,
  };
  pub use datafusion::arrow::record_batch::RecordBatch;
  pub use datafusion::catalog::{
    CatalogList as DatafusionCatalogList,
    CatalogProvider as DatafusionCatalogProvider,
  };
  pub use datafusion::common::{
    config::ConfigOptions, DFSchema, ScalarType, ScalarValue, TableReference,
  };
  pub use datafusion::error::{DataFusionError, Result};
  pub use datafusion::execution::{context::SessionState, TaskContext};
  pub use datafusion::logical_expr::expr::Expr;
  pub use datafusion::logical_expr::{
    create_udf, AggregateUDF, LogicalPlan, ScalarUDF, Volatility, WindowUDF,
  };
  pub use datafusion::physical_plan::{
    ColumnarValue, SendableRecordBatchStream as RecordBatchStream,
  };
}

pub mod response {
  pub use crate::execution::response::*;
}

// Re-exports
pub use bytes;
pub use postgres_types;

pub mod arrow {
  pub use datafusion::arrow::array::{
    as_boolean_array, as_generic_list_array, as_null_array, as_primitive_array,
    as_string_array, Array, ArrayAccessor, ArrayIter, ArrayRef, BinaryArray,
    BinaryBuilder, BooleanArray, BooleanBuilder, Float32Array, Float32Builder,
    Float64Array, Float64Builder, Int16Array, Int32Array, Int32Builder,
    Int64Array, Int64Builder, ListArray, NullBuilder, StringArray,
    StringBuilder, UInt16Array, UInt32Array, UInt64Array,
  };
  pub use datafusion::arrow::datatypes::{
    Float32Type, Float64Type, Int32Type, TimeUnit, TimestampNanosecondType,
    UInt32Type, UInt64Type,
  };
  pub use datafusion::common::cast::as_binary_array;
}
