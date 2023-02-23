use crate::utils::fs::resolve_read_path;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::StringOrBuffer;
use deno_core::ZeroCopyBuf;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub fn init() -> Extension {
  Extension::builder("<arena/core/fs>")
    .ops(vec![
      op_read_file_sync::decl(),
      op_read_file_async::decl(),
      op_read_file_string_async::decl(),
    ])
    .build()
}

#[op]
fn op_read_file_sync(
  state: &mut OpState,
  specifier: String,
) -> Result<ZeroCopyBuf> {
  let resolved_path = resolve_read_path(state, &Path::new(&specifier))?;
  Ok(std::fs::read(resolved_path)?.into())
}

#[op]
async fn op_read_file_async(
  state: Rc<RefCell<OpState>>,
  specifier: String,
) -> Result<ZeroCopyBuf> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&specifier))
  }?;
  Ok(tokio::fs::read(resolved_path).await?.into())
}

#[op]
async fn op_read_file_string_async(
  state: Rc<RefCell<OpState>>,
  specifier: String,
) -> Result<StringOrBuffer> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&specifier))
  }?;
  Ok(StringOrBuffer::String(
    tokio::fs::read_to_string(resolved_path).await?,
  ))
}
