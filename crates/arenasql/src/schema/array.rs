use std::sync::Arc;

use datafusion::arrow::array::{
  ArrayRef, BinaryBuilder, BooleanBuilder, Float32Builder, Float64Builder,
  Int16Builder, Int32Builder, Int64Builder, ListBuilder, StringBuilder,
  UInt32Builder, UInt64Builder,
};

use super::{DataType, SerializedCell};

pub enum ColumnArrayBuilder {
  Boolean(BooleanBuilder),
  Int16(Int16Builder),
  Int32(Int32Builder),
  UInt32(UInt32Builder),
  Int64(Int64Builder),
  UInt64(UInt64Builder),
  Float32(Float32Builder),
  Float64(Float64Builder),
  String(StringBuilder),
  Binary(BinaryBuilder),
  Vector(ListBuilder<Float32Builder>),
}

impl ColumnArrayBuilder {
  pub fn from(data_type: &DataType, capacity: usize) -> ColumnArrayBuilder {
    match data_type {
      DataType::Boolean => {
        ColumnArrayBuilder::Boolean(BooleanBuilder::with_capacity(capacity))
      }
      DataType::Int16 => {
        ColumnArrayBuilder::Int16(Int16Builder::with_capacity(capacity))
      }
      DataType::Int32 => {
        ColumnArrayBuilder::Int32(Int32Builder::with_capacity(capacity))
      }
      DataType::UInt32 => {
        ColumnArrayBuilder::UInt32(UInt32Builder::with_capacity(capacity))
      }
      DataType::Int64 => {
        ColumnArrayBuilder::Int64(Int64Builder::with_capacity(capacity))
      }
      DataType::UInt64 => {
        ColumnArrayBuilder::UInt64(UInt64Builder::with_capacity(capacity))
      }
      DataType::Float32 => {
        ColumnArrayBuilder::Float32(Float32Builder::with_capacity(capacity))
      }
      DataType::Float64 => {
        ColumnArrayBuilder::Float64(Float64Builder::with_capacity(capacity))
      }
      DataType::Text => ColumnArrayBuilder::String(
        StringBuilder::with_capacity(capacity, capacity * 250),
      ),
      DataType::Varchar { len } => {
        ColumnArrayBuilder::String(StringBuilder::with_capacity(
          capacity,
          capacity * len.unwrap_or(1000) as usize,
        ))
      }
      DataType::Binary => {
        ColumnArrayBuilder::Binary(BinaryBuilder::with_capacity(capacity, 1000))
      }
      DataType::Vector { len } => {
        ColumnArrayBuilder::Vector(ListBuilder::with_capacity(
          Float32Builder::with_capacity(*len),
          capacity,
        ))
      }
      v => unimplemented!("Not implemented for data type: {:?}", v),
    }
  }

  #[inline]
  pub fn append(&mut self, value: &SerializedCell<'_>) {
    match self {
      Self::Boolean(ref mut builder) => builder.append_option(value.as_bool()),
      Self::Int16(ref mut builder) => builder.append_option(value.as_i16()),
      Self::Int32(ref mut builder) => builder.append_option(value.as_i32()),
      Self::UInt32(ref mut builder) => builder.append_option(value.as_u32()),
      Self::Int64(ref mut builder) => builder.append_option(value.as_i64()),
      Self::UInt64(ref mut builder) => builder.append_option(value.as_u64()),
      Self::Float32(ref mut builder) => builder.append_option(value.as_f32()),
      Self::Float64(ref mut builder) => builder.append_option(value.as_f64()),
      Self::String(ref mut builder) => builder.append_option(value.as_str()),
      Self::Binary(ref mut builder) => builder.append_option(value.as_bytes()),
      Self::Vector(ref mut builder) => {
        let vector = value.as_vector().unwrap();
        builder.append_option(Some(vector.clone().iter().map(|f| Some(*f))))
      }
    }
  }

  #[inline]
  pub fn finish(self) -> ArrayRef {
    match self {
      Self::Boolean(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Int16(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Int32(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::UInt32(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Int64(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::UInt64(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Float32(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Float64(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::String(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Binary(mut v) => Arc::new(v.finish()) as ArrayRef,
      Self::Vector(mut v) => Arc::new(v.finish()) as ArrayRef,
    }
  }
}
