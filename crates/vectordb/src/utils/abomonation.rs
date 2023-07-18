use abomonation::Abomonation;
use anyhow::{anyhow, Result};

pub fn decode<'a, T>(bytes: &'a mut [u8]) -> Result<&'a T>
where
  T: Abomonation,
{
  let (embedding, remaining) = unsafe { abomonation::decode::<T>(bytes) }
    .ok_or(anyhow!("Error decoding embeddings"))?;
  assert!(remaining.len() == 0);
  Ok(embedding)
}

pub fn encode<T>(embedding: &T) -> Result<Vec<u8>>
where
  T: Abomonation,
{
  let mut encoded_embeddings = Vec::new();
  unsafe {
    abomonation::encode(embedding, &mut encoded_embeddings)?;
  };
  Ok(encoded_embeddings)
}
