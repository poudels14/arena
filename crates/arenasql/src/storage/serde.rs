use bincode::{DefaultOptions, Error, Options};

#[inline(always)]
pub fn serialize<S: ?Sized + serde::Serialize>(
  data: &S,
) -> Result<Vec<u8>, Error> {
  DefaultOptions::default().with_big_endian().serialize(data)
}

#[inline(always)]
pub fn deserialize<'a, T: serde::Deserialize<'a>>(
  bytes: &'a [u8],
) -> Result<T, Error> {
  DefaultOptions::default()
    .with_big_endian()
    .deserialize(bytes)
}
