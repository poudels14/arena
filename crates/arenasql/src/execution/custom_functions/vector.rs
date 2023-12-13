use std::sync::Arc;

use datafusion::arrow::array::{
  Array, Float32Array, Float32Builder, ListArray,
};
use datafusion::arrow::datatypes::{DataType, Field};
use datafusion::error::Result;
use datafusion::logical_expr::{create_udf, ScalarUDF, Volatility};
use datafusion::physical_plan::ColumnarValue;
use datafusion::scalar::ScalarValue;
use once_cell::sync::Lazy;

use crate::vectors::{SimilarityScorerFactory, SimilarityType};
use crate::{bail, df_error, Error};

macro_rules! invalid_query {
  ($msg:expr) => {
    df_error!(Error::InvalidQuery(format!($msg)))
  };
}

pub const L2_DISTANCE: Lazy<ScalarUDF> = Lazy::new(|| {
  create_udf(
    "l2",
    vec![
      DataType::List(Field::new("item", DataType::Float32, true).into()),
      // JSON string of vector array
      DataType::Utf8,
    ],
    Arc::new(DataType::Float32),
    Volatility::Immutable,
    Arc::new(l2_distance),
  )
});

pub fn l2_distance(args: &[ColumnarValue]) -> Result<ColumnarValue> {
  let ColumnarValue::Array(ref vector_column) = args[0] else {
    return Err(invalid_query!(
      "First argument of \"L2\" should be a vector column"
    ));
  };
  let ColumnarValue::Scalar(ScalarValue::Utf8(Some(ref raw_input_vector))) =
    args[1]
  else {
    return Err(invalid_query!(
      "Second argument of \"L2\" should be a valid JSON string"
    ));
  };

  let input_vector = serde_json::from_str::<Vec<f32>>(&raw_input_vector)
    .map_err(|_| {
      invalid_query!(
        "Second argument of \"L2\" should be a valid JSON string of a vector"
      )
    })?;

  let vector_array =
    vector_column.as_any().downcast_ref::<ListArray>().unwrap();

  // Short circuit if column vector is empty
  if vector_array.len() == 0 {
    return Ok(ColumnarValue::Array(Arc::<Float32Array>::new(
      Vec::<f32>::new().into(),
    )));
  }

  let vector_length = vector_array.len();
  if vector_length != input_vector.len() {
    bail!(invalid_query!(
      "Input vector length doesn't match column vector length"
    ));
  }

  let entire_vector_array = vector_array
    .values()
    .slice(0, vector_length * vector_array.len());

  let vector_data = entire_vector_array.to_data();
  let vector_buffers = vector_data.buffers();
  if vector_buffers.len() > 1 {
    panic!("TODO: handle more than 1 vector buffer");
  }

  let vectors = vector_data.buffer::<f32>(0);
  let scorer = SimilarityScorerFactory::get_default(SimilarityType::Dot);

  let mut similarity_scores = Float32Builder::with_capacity(vector_array.len());
  for i in 0..vector_array.len() {
    let score = scorer.similarity_score(
      &vectors[i * vector_length..(i + 1) * vector_length],
      &input_vector,
    );
    similarity_scores.append_value(score.0);
  }

  Ok(ColumnarValue::Array(Arc::<Float32Array>::new(
    similarity_scores.finish(),
  )))
}
