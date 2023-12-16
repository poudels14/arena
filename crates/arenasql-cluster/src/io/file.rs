use std::io::Write;
use std::path::Path;

use anyhow::Result;

use super::serializer::Serializer;

pub struct File {
  inner: std::fs::File,
}

impl File {
  pub fn create(path: &Path) -> Result<Self> {
    let inner = std::fs::File::create(path)?;
    Ok(Self { inner })
  }

  pub fn read<T>(path: &Path) -> Result<T>
  where
    T: serde::de::DeserializeOwned,
  {
    let file = std::fs::File::open(path)?;
    let value = Serializer::default().deserialize_from(file)?;
    Ok(value)
  }

  pub fn write_sync<T: ?Sized>(&mut self, value: &T) -> Result<()>
  where
    T: serde::Serialize,
  {
    let bytes = Serializer::default().serialize(value)?;
    self.inner.write_all(&bytes)?;
    self.inner.sync_all()?;
    Ok(())
  }
}
