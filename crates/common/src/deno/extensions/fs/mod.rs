use super::super::utils::fs::{resolve_read_path, resolve_write_path};
use super::BuiltinExtension;
use anyhow::anyhow;
use anyhow::Result;
use deno_core::{op2, JsBuffer, OpState, ToJsBuffer};
use serde_json::json;
use std::cell::RefCell;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::fs::init_ops_and_esm()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

deno_core::extension!(
  fs,
  ops = [
    op_fs_cwd_sync,
    op_fs_lstat_sync,
    op_fs_realpath_sync,
    op_fs_readdir_sync,
    op_fs_file_exists_sync,
    op_fs_mkdir_sync,
    op_fs_read_file_sync,
    op_fs_read_file_async,
    op_fs_read_file_string_async,
    op_fs_write_file_sync,
  ],
  js = [dir "src/deno/extensions/fs", "fs.js"]
);

#[op2]
#[string]
fn op_fs_cwd_sync(state: &mut OpState) -> Result<String> {
  let resolved_path = resolve_read_path(state, &Path::new("."))?;
  resolved_path
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[op2]
#[serde]
fn op_fs_lstat_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<serde_json::Value> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  let m = std::fs::metadata(resolved_path)?;
  Ok(json!({
    "dev": m.dev(),
    "ino": m.ino(),
    "mode": m.mode(),
    "nlink": m.nlink(),
    "uid": m.uid(),
    "gid": m.gid(),
    "rdev": m.rdev(),
    "size": m.size(),
    "blksize": m.blksize(),
    "blocks": m.blocks(),
    "atimeMs": m.atime_nsec() / 1000,
    "mtimeMs":  m.mtime_nsec() / 1000,
    "ctimeMs": m.ctime_nsec() / 1000,
    "birthtimeMs": m.created()?
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_millis(),
    "isSymlink": m.is_symlink(),
    "isFile": m.is_file(),
  }))
}

#[op2]
#[string]
fn op_fs_realpath_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<String> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  resolved_path
    .canonicalize()?
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[op2]
#[serde]
fn op_fs_readdir_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<Vec<String>> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  Ok(
    resolved_path
      .read_dir()?
      .flatten()
      .map(|dir| dir.file_name().to_str().unwrap().to_owned())
      .collect(),
  )
}

#[op2(fast)]
fn op_fs_file_exists_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<bool> {
  resolve_read_path(state, &Path::new(&path)).map(|f| f.exists())
}

#[op2(fast)]
fn op_fs_mkdir_sync(
  state: &mut OpState,
  #[string] path: String,
  recursive: bool,
) -> Result<()> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  match recursive {
    true => std::fs::create_dir_all(resolved_path),
    false => std::fs::create_dir(resolved_path),
  }
  .map_err(|e| anyhow!("{}", e))
}

#[op2]
#[serde]
fn op_fs_read_file_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<ToJsBuffer> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  Ok(std::fs::read(resolved_path)?.into())
}

#[op2(async)]
#[serde]
async fn op_fs_read_file_async(
  state: Rc<RefCell<OpState>>,
  #[string] path: String,
) -> Result<ToJsBuffer> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&path))
  }?;

  Ok(tokio::fs::read(resolved_path).await?.into())
}

#[op2(async)]
#[string]
async fn op_fs_read_file_string_async(
  state: Rc<RefCell<OpState>>,
  #[string] path: String,
) -> Result<String> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&path))
  }?;
  Ok(tokio::fs::read_to_string(resolved_path).await?)
}

#[op2(fast)]
fn op_fs_write_file_sync(
  state: &mut OpState,
  #[string] path: String,
  #[buffer] data: JsBuffer,
) -> Result<()> {
  let resolved_path = resolve_write_path(state, &Path::new(&path))?;
  let mut file = std::fs::File::create(resolved_path)?;
  file.write_all(data.as_ref()).map_err(|e| anyhow!("{}", e))
}
