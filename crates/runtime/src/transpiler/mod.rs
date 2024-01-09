mod babel;
mod swc;

pub mod jsx_analyzer;
use anyhow::Result;
use async_trait::async_trait;
pub use babel::BabelTranspiler;
pub use swc::{with_esm_exports, SwcTranspiler};

use std::path::PathBuf;
use std::sync::Arc;

#[async_trait]
pub trait ModuleTranspiler {
  async fn transpile<'a>(
    &'a self,
    path: &PathBuf,
    code: &str,
  ) -> Result<Arc<str>>;
}
