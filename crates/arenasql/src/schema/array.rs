use std::sync::Arc;

use datafusion::arrow::array::{
  ArrayRef, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Builder,
  Int32Builder, Int64Builder, NullBuilder, StringBuilder,
};

use super::{DataType, SerializedCell};

#[derive(Debug)]
pub enum ColumnArrayBuilder {
  // No op - this is used when the column isn't selected
  // makes it easier to add row cells to array
  Noop,
  Null(NullBuilder),
  Boolean(BooleanBuilder),
  Int32(Int32Builder),
  Int64(Int64Builder),
  Float32(Float32Builder),
  Float64(Float64Builder),
  String(StringBuilder),
  Binary(BinaryBuilder),
}

impl ColumnArrayBuilder {
  pub fn from(data_type: &DataType, capacity: usize) -> ColumnArrayBuilder {
    match data_type {
      DataType::Boolean => {
        ColumnArrayBuilder::Boolean(BooleanBuilder::with_capacity(capacity))
      }

      DataType::Int32 => {
        ColumnArrayBuilder::Int32(Int32Builder::with_capacity(capacity))
      }
      DataType::Int64 => {
        ColumnArrayBuilder::Int64(Int64Builder::with_capacity(capacity))
      }

      DataType::Float32 => {
        ColumnArrayBuilder::Float32(Float32Builder::with_capacity(capacity))
      }
      DataType::Float64 => {
        ColumnArrayBuilder::Float64(Float64Builder::with_capacity(capacity))
      }
      DataType::Text => ColumnArrayBuilder::String(
        StringBuilder::with_capacity(capacity, capacity * 1000),
      ),
      DataType::Varchar { len } => ColumnArrayBuilder::String(
        StringBuilder::with_capacity(capacity, capacity * *len as usize),
      ),
      DataType::Binary => {
        ColumnArrayBuilder::Binary(BinaryBuilder::with_capacity(capacity, 1000))
      }
      v => unimplemented!("Not implemented for data type: {:?}", v),
    }
  }

  #[inline]
  pub fn append(&mut self, value: &SerializedCell<&[u8]>) {
    match self {
      Self::Noop => {}
      Self::Boolean(ref mut builder) => builder.append_option(value.as_bool()),
      Self::Int32(ref mut builder) => builder.append_option(value.as_i32()),
      Self::Int64(ref mut builder) => builder.append_option(value.as_i64()),
      Self::Float32(ref mut builder) => builder.append_option(value.as_f32()),
      Self::Float64(ref mut builder) => builder.append_option(value.as_f64()),
      Self::String(ref mut builder) => builder.append_option(value.as_str()),
      Self::Binary(ref mut builder) => builder.append_option(value.as_bytes()),
      v => unimplemented!("Not implemented for data type: {:?}", v),
    }
  }

  #[inline]
  pub fn finish(self) -> ArrayRef {
    match self {
      Self::Int32(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::String(mut v) => Arc::new(v.finish()) as ArrayRef,
      _ => unimplemented!(),
    }
  }
}
