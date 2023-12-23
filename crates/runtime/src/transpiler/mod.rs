use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

mod babel;
mod transpiler;

pub use babel::BabelTranspiler;
use deno_ast::MediaType;

#[async_trait]
pub trait ModuleTranspiler {
  async fn transpile<'a>(
    &'a self,
    path: &PathBuf,
    media_type: &MediaType,
    code: &str,
  ) -> Result<Arc<str>>;
}
