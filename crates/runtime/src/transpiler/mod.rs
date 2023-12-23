use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

mod babel;

pub use babel::BabelTranspiler;

#[async_trait]
pub trait ModuleTranspiler {
  async fn transpile<'a>(
    &'a self,
    path: &PathBuf,
    code: &str,
  ) -> Result<Arc<str>>;
}
