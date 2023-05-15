use super::super::permissions::PermissionsContainer;
use anyhow::bail;
use anyhow::Result;
use deno_core::normalize_path;
use deno_core::OpState;
use std::path::Path;
use std::path::PathBuf;

#[inline]
fn resolve(base: &Path, path: &Path) -> Result<PathBuf> {
  if path.is_absolute() {
    Ok(normalize_path(path))
  } else {
    Ok(normalize_path(base.join(path)))
  }
}

/// resolves the given path from project prefix/root and checks for
/// read permission. Returns Ok(resolved_path) if the permission
/// for given path is granted, else returns error
#[inline]
#[allow(dead_code)]
pub fn resolve_read_path(state: &mut OpState, path: &Path) -> Result<PathBuf> {
  let permissions = state.borrow_mut::<PermissionsContainer>();

  match permissions.fs.as_ref() {
    Some(perm) => {
      let resolved_path = resolve(&perm.root, path)?;
      permissions.check_read(&resolved_path)?;
      Ok(resolved_path)
    }
    None => bail!("No access to filesystem"),
  }
}

/// resolves the given path from project prefix/root and checks for
/// write permission. Returns Ok(resolved_path) if the permission
/// for given path is granted, else returns error
#[inline]
#[allow(dead_code)]
pub fn resolve_write_path(state: &mut OpState, path: &Path) -> Result<PathBuf> {
  let permissions = state.borrow_mut::<PermissionsContainer>();

  match permissions.fs.as_ref() {
    Some(perm) => {
      let resolved_path = resolve(&perm.root, path)?;
      permissions.check_write(&resolved_path)?;
      Ok(resolved_path)
    }
    None => bail!("No access to filesystem"),
  }
}

/// This macro returns the absolute path of the file that is
/// relative to the project
/// |true| should be passed as second arg in build scripts to rerun
/// build when the file changes BUT it's not needed when using the
/// macro inside |common| module
#[macro_export]
macro_rules! resolve_from_root {
  ($a:expr) => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      resolve_from_root!($a, true)
    }
    #[cfg(not(feature = "fs_rerun_if_changed"))]
    {
      resolve_from_root!($a, false)
    }
  }};
  ($a:expr, $b: literal) => {{
    use std::path::PathBuf;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .join($a)
      .canonicalize()
      .unwrap();

    if $b {
      let path = path.to_str().unwrap();
      println!("cargo:rerun-if-changed={}", path);
    }
    path
  }};
}

/// This macro returns the absolute path of the file that is
/// relative to the current file
/// |true| should be passed as second arg in build scripts to rerun
/// build when the file changes BUT it's not needed when using the
/// macro inside |common| module
#[macro_export]
macro_rules! resolve_from_file {
  ($a:expr) => {{
    #[cfg(feature = "fs-rerun-if-changed")]
    {
      resolve_from_file!($a, true)
    }
    #[cfg(not(feature = "fs-rerun-if-changed"))]
    {
      resolve_from_file!($a, false)
    }
  }};
  ($a:expr, $b: literal) => {{
    use std::path::PathBuf;
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
      .parent()
      .unwrap()
      .join(file!())
      .parent()
      .unwrap()
      .join($a)
      .canonicalize()
      .unwrap();

    if $b {
      let path = path.to_str().unwrap();
      println!("cargo:rerun-if-changed={}", path);
    }
    path
  }};
}
