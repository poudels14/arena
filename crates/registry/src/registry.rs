use crate::cache::FileStorage;
use anyhow::Result;
use mime::Mime;

#[derive(Clone)]
pub struct Registry {
  /// List of directories that contains cached JS files
  cache: FileStorage,
}

pub struct FileContent {
  pub mime: Mime,
  pub content: Vec<u8>,
}

impl Registry {
  pub fn with_cache(cache: FileStorage) -> Self {
    Self { cache }
  }
}

impl Registry {
  pub async fn get_contents(
    &self,
    uri: &str,
  ) -> Result<Option<Box<FileContent>>> {
    self.cache.get_contents(uri).await
  }
}
