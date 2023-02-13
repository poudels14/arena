use crate::permissions::PermissionsContainer;
use anyhow::Context;
use anyhow::Result;
use deno_core::normalize_path;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::ZeroCopyBuf;
use std::env::current_dir;
use std::path::Path;
use std::path::PathBuf;

pub fn init() -> Extension {
  Extension::builder("<arena/core/fs>")
    .ops(vec![op_read_file_sync::decl()])
    .build()
}

#[op]
pub fn op_read_file_sync(
  state: &mut OpState,
  specifier: String,
) -> Result<ZeroCopyBuf> {
  let resolved_path = resolve_from_cwd(&Path::new(&specifier))?;

  let permissions = state.borrow_mut::<PermissionsContainer>();
  permissions.check_read(&resolved_path)?;

  Ok(std::fs::read(resolved_path)?.into())
}

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
