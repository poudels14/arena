use crate::utils::fs::resolve_read_path;
use crate::utils::fs::resolve_write_path;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use deno_core::StringOrBuffer;
use serde_json::json;
use serde_json::Value;
use std::cell::RefCell;
use std::io::Write;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::rc::Rc;
use std::time::SystemTime;

pub fn init() -> Extension {
  Extension::builder("arena/fs")
    .ops(vec![
      op_fs_cwd_sync::decl(),
      op_fs_lstat_sync::decl(),
      op_fs_realpath_sync::decl(),
      op_fs_readdir_sync::decl(),
      op_fs_file_exists_sync::decl(),
      op_fs_mkdir_sync::decl(),
      op_fs_read_file_sync::decl(),
      op_fs_read_file_async::decl(),
      op_fs_read_file_string_async::decl(),
      op_fs_write_file_sync::decl(),
    ])
    .js(vec![ExtensionFileSource {
      specifier: "setup".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!("./fs.js")),
    }])
    .build()
}

#[op(fast)]
fn op_fs_cwd_sync(state: &mut OpState) -> Result<String> {
  let resolved_path = resolve_read_path(state, &Path::new("."))?;
  resolved_path
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[op(fast)]
fn op_fs_lstat_sync(state: &mut OpState, path: String) -> Result<Value> {
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

#[op(fast)]
fn op_fs_realpath_sync(state: &mut OpState, path: String) -> Result<String> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  resolved_path
    .canonicalize()?
    .to_str()
    .map(|s| s.to_owned())
    .ok_or(anyhow!("Couldn't get current directory"))
}

#[op(fast)]
fn op_fs_readdir_sync(
  state: &mut OpState,
  path: String,
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

#[op(fast)]
fn op_fs_file_exists_sync(state: &mut OpState, path: String) -> Result<bool> {
  Ok(resolve_read_path(state, &Path::new(&path))?.exists())
}

#[op(fast)]
fn op_fs_mkdir_sync(
  state: &mut OpState,
  path: String,
  recursive: bool,
) -> Result<()> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  match recursive {
    true => std::fs::create_dir_all(resolved_path),
    false => std::fs::create_dir(resolved_path),
  }
  .map_err(|e| anyhow!("{}", e))
}

#[op(fast)]
fn op_fs_read_file_sync(
  state: &mut OpState,
  path: String,
  encoding: Option<String>,
) -> Result<StringOrBuffer> {
  let resolved_path = resolve_read_path(state, &Path::new(&path))?;
  match encoding.as_deref() {
    Some("utf8") | Some("utf-8") => Ok(StringOrBuffer::String(
      std::fs::read_to_string(resolved_path)?,
    )),
    None => Ok(StringOrBuffer::Buffer(std::fs::read(resolved_path)?.into())),
    Some(e) => bail!("Unsupported encoding: {:?}", e),
  }
}

#[op]
async fn op_fs_read_file_async(
  state: Rc<RefCell<OpState>>,
  path: String,
  encoding: Option<String>,
) -> Result<StringOrBuffer> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&path))
  }?;

  match encoding.as_deref() {
    Some("utf8") | Some("utf-8") => Ok(StringOrBuffer::String(
      tokio::fs::read_to_string(resolved_path).await?,
    )),
    None => Ok(StringOrBuffer::Buffer(
      tokio::fs::read(resolved_path).await?.into(),
    )),
    Some(e) => bail!("Unsupported encoding: {:?}", e),
  }
}

#[op]
async fn op_fs_read_file_string_async(
  state: Rc<RefCell<OpState>>,
  path: String,
) -> Result<StringOrBuffer> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&path))
  }?;
  Ok(StringOrBuffer::String(
    tokio::fs::read_to_string(resolved_path).await?,
  ))
}

#[op(fast)]
fn op_fs_write_file_sync(
  state: &mut OpState,
  path: String,
  data: StringOrBuffer,
) -> Result<()> {
  let resolved_path = resolve_write_path(state, &Path::new(&path))?;
  let mut file = std::fs::File::create(resolved_path)?;
  file.write_all(data.as_ref()).map_err(|e| anyhow!("{}", e))
}
