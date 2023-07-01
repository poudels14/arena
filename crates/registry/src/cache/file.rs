use crate::registry::FileContent;
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct FileStorage {
  /// A directory that contains cached files
  dir: PathBuf,
}

impl FileStorage {
  pub fn new(dir: &Path) -> Self {
    Self {
      dir: dir.to_owned(),
    }
  }
}

impl FileStorage {
  pub async fn get_contents(
    &self,
    uri: &str,
  ) -> Result<Option<Box<FileContent>>> {
    let path = self.dir.join(uri);
    if path.exists() {
      let mut file = File::open(&path).await?;
      let mut contents = vec![];
      let _ = file.read_to_end(&mut contents).await?;

      return Ok(Some(
        FileContent {
          mime: mime_guess::from_path(path).first_or_text_plain(),
          content: contents.into(),
        }
        .into(),
      ));
    }
    Ok(None)
  }
}
