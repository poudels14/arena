use anyhow::Context;
use anyhow::Result;
use deno_core::normalize_path;
use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;

#[inline]
pub fn resolve_from_cwd(path: &Path) -> Result<PathBuf> {
  if path.is_absolute() {
    Ok(normalize_path(path))
  } else {
    let cwd =
      current_dir().context("Failed to get current working directory")?;
    Ok(normalize_path(cwd.join(path)))
  }
}
