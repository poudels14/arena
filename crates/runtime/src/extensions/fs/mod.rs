use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use bitflags::bitflags;
use deno_core::Resource;
use deno_core::ResourceId;
use deno_core::{op2, JsBuffer, OpState, ToJsBuffer};
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use std::fs::File;
use std::fs::Metadata;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

use super::BuiltinExtension;
use crate::permissions;

bitflags! {
  // This should match with nodejs "fs" module
  // check 'constants-browserify' for reference
  #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
  struct Flag: u32 {
    const F_OK = 0;
    const X_OK = 1;
    const W_OK = 2;
    const R_OK = 4;
  }
}

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(Some(self::fs::init_ops_and_esm()), vec![])
}

deno_core::extension!(
  fs,
  ops = [
    op_fs_cwd_sync,
    op_fs_access_sync,
    op_fs_lstat_sync,
    op_fs_stat_sync,
    op_fs_realpath_sync,
    op_fs_open_sync,
    op_fs_close_sync,
    op_fs_readdir_sync,
    op_fs_file_exists_sync,
    op_fs_mkdir_sync,
    op_fs_read_file_sync,
    op_fs_read_file_async,
    op_fs_read_file_string_async,
    op_fs_write_file_sync,
  ],
  js = [dir "src/extensions/fs", "fs.js"]
);

#[allow(unused)]
struct FileResource {
  file: File,
}

impl Resource for FileResource {}

#[tracing::instrument(skip(state), ret, level = "trace")]
#[op2]
#[string]
fn op_fs_cwd_sync(state: &mut OpState) -> Result<String> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new("."))?;
  resolved_path
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[tracing::instrument(skip(state), ret, level = "trace")]
#[op2(fast)]
fn op_fs_access_sync(
  state: &mut OpState,
  #[string] path: String,
  #[smi] flag: u32,
) -> Result<()> {
  let flag = Flag::from_bits(flag).unwrap_or(Flag::F_OK);
  match flag {
    Flag::F_OK | Flag::R_OK => {
      permissions::resolve_read_path(state, &Path::new(&path)).map(|_| ())
    }
    Flag::W_OK => {
      permissions::resolve_write_path(state, &Path::new(&path)).map(|_| ())
    }
    // Can't execute any file
    Flag::X_OK | _ => {
      bail!("No access")
    }
  }
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2]
#[serde]
fn op_fs_lstat_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<serde_json::Value> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  let m = std::fs::symlink_metadata(resolved_path)?;
  to_stat_json(m)
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2]
#[serde]
fn op_fs_stat_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<serde_json::Value> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  let m = std::fs::metadata(resolved_path)?;
  to_stat_json(m)
}

fn to_stat_json(m: Metadata) -> Result<Value> {
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
    "isDirectory": m.is_dir(),
  }))
}

#[tracing::instrument(skip(state), ret, level = "trace")]
#[op2]
#[string]
fn op_fs_realpath_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<String> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  resolved_path
    .canonicalize()?
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(fast)]
#[smi]
fn op_fs_open_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<ResourceId> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  let file = File::open(resolved_path)?;
  let rid = state
    .resource_table
    .add::<FileResource>(FileResource { file });
  Ok(rid)
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(fast)]
fn op_fs_close_sync(state: &mut OpState, #[smi] rid: ResourceId) -> Result<()> {
  state.resource_table.take::<FileResource>(rid)?;
  Ok(())
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2]
#[serde]
fn op_fs_readdir_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<Vec<serde_json::Value>> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;

  resolved_path
    .read_dir()?
    .flatten()
    .map(|dir| {
      let m = dir.metadata()?;
      Ok(json!({
        "name": dir.file_name().to_str().unwrap(),
        "isSymlink": m.is_symlink(),
        "isFile": m.is_file(),
        "isDirectory": m.is_dir(),
      }))
    })
    .collect()
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(fast)]
fn op_fs_file_exists_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<bool> {
  permissions::resolve_read_path(state, &Path::new(&path)).map(|f| f.exists())
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(fast)]
fn op_fs_mkdir_sync(
  state: &mut OpState,
  #[string] path: String,
  recursive: bool,
) -> Result<()> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  match recursive {
    true => std::fs::create_dir_all(resolved_path),
    false => std::fs::create_dir(resolved_path),
  }
  .map_err(|e| anyhow!("{}", e))
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2]
#[serde]
fn op_fs_read_file_sync(
  state: &mut OpState,
  #[string] path: String,
) -> Result<ToJsBuffer> {
  let resolved_path = permissions::resolve_read_path(state, &Path::new(&path))?;
  Ok(std::fs::read(resolved_path)?.into())
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(async)]
#[serde]
async fn op_fs_read_file_async(
  state: Rc<RefCell<OpState>>,
  #[string] path: String,
) -> Result<ToJsBuffer> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    permissions::resolve_read_path(&mut state, &Path::new(&path))
  }?;

  Ok(tokio::fs::read(resolved_path).await?.into())
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2(async)]
#[string]
async fn op_fs_read_file_string_async(
  state: Rc<RefCell<OpState>>,
  #[string] path: String,
) -> Result<String> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    permissions::resolve_read_path(&mut state, &Path::new(&path))
  }?;
  Ok(tokio::fs::read_to_string(resolved_path).await?)
}

#[tracing::instrument(skip(state, data), level = "trace")]
#[op2(fast)]
fn op_fs_write_file_sync(
  state: &mut OpState,
  #[string] path: String,
  #[buffer] data: JsBuffer,
) -> Result<()> {
  let resolved_path =
    permissions::resolve_write_path(state, &Path::new(&path))?;
  let mut file = std::fs::File::create(resolved_path)?;
  file.write_all(data.as_ref()).map_err(|e| anyhow!("{}", e))
}
