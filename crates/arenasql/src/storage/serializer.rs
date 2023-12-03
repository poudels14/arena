use bincode::config::{
  BigEndian, FixintEncoding, VarintEncoding, WithOtherEndian,
  WithOtherIntEncoding,
};
use bincode::DefaultOptions;
pub use bincode::Options as SerializerOptions;
use once_cell::sync::Lazy;

use crate::Result;

const VARINT_SERIALIZER: Lazy<
  WithOtherIntEncoding<
    WithOtherEndian<DefaultOptions, BigEndian>,
    VarintEncoding,
  >,
> = Lazy::new(|| {
  DefaultOptions::default()
    .with_big_endian()
    .with_varint_encoding()
});

const FIXINT_SERIALIZER: Lazy<
  WithOtherIntEncoding<
    WithOtherEndian<DefaultOptions, BigEndian>,
    FixintEncoding,
  >,
> = Lazy::new(|| {
  DefaultOptions::default()
    .with_big_endian()
    .with_fixint_encoding()
});

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Serializer {
  VarInt = 1,
  FixedInt = 2,
}

impl Serializer {
  pub fn serialize<S: ?Sized + serde::Serialize>(
    &self,
    data: &S,
  ) -> Result<Vec<u8>>
  where
    Self: Sized,
  {
    match self {
      Self::VarInt => Ok(VARINT_SERIALIZER.serialize(data)?),
      Self::FixedInt => Ok(FIXINT_SERIALIZER.serialize(data)?),
    }
  }

  pub fn deserialize<'a, T: serde::Deserialize<'a>>(
    &self,
    bytes: &'a [u8],
  ) -> Result<T>
  where
    Self: Sized,
  {
    match self {
      Self::VarInt => Ok(VARINT_SERIALIZER.deserialize(bytes)?),
      Self::FixedInt => Ok(FIXINT_SERIALIZER.deserialize(bytes)?),
    }
  }

  pub fn deserialize_or_log_error<'a, T: serde::Deserialize<'a>>(
    &self,
    bytes: &'a [u8],
  ) -> Option<T>
  where
    Self: Sized,
  {
    match self.deserialize::<T>(bytes) {
      Ok(v) => Some(v),
      Err(e) => {
        eprintln!("Error deserializing: {:?}", e);
        return None;
      }
    }
  }
}

impl Default for Serializer {
  fn default() -> Self {
    Self::VarInt
  }
}
