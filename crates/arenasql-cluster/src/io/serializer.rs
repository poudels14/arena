use std::io::Read;

use anyhow::Result;
use bincode::config::{
  BigEndian, FixintEncoding, VarintEncoding, WithOtherEndian,
  WithOtherIntEncoding,
};
use bincode::DefaultOptions;
pub use bincode::Options as SerializerOptions;
use once_cell::sync::Lazy;

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

#[allow(unused)]
#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Serializer {
  VarInt = 1,
  FixedInt = 2,
}

impl Serializer {
  #[inline]
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

  #[allow(unused)]
  #[inline]
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

  #[inline]
  pub fn deserialize_from<R: Read, T: serde::de::DeserializeOwned>(
    self,
    reader: R,
  ) -> Result<T> {
    match self {
      Self::VarInt => Ok(VARINT_SERIALIZER.deserialize_from(reader)?),
      Self::FixedInt => Ok(FIXINT_SERIALIZER.deserialize_from(reader)?),
    }
  }
}

impl Default for Serializer {
  fn default() -> Self {
    Self::VarInt
  }
}
