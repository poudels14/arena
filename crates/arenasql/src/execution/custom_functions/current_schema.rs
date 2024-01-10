use std::sync::Arc;

use datafusion::arrow::datatypes::DataType;
use datafusion::error::Result;
use datafusion::logical_expr::{create_udf, ScalarUDF, Volatility};
use datafusion::physical_plan::ColumnarValue;
use datafusion::scalar::ScalarValue;
use once_cell::sync::Lazy;

pub const CURRENT_SCHEMA: Lazy<ScalarUDF> = Lazy::new(|| {
  create_udf(
    "current_schema",
    vec![],
    Arc::new(DataType::Utf8),
    Volatility::Immutable,
    Arc::new(current_schema),
  )
});

pub fn current_schema(_args: &[ColumnarValue]) -> Result<ColumnarValue> {
  Ok(ColumnarValue::Scalar(ScalarValue::Utf8(Some(
    "public".to_owned(),
  ))))
}
