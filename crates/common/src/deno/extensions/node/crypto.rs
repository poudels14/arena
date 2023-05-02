// credit: deno
// Copyright 2018-2023 the Deno authors. All rights reserved. MIT license.
use super::digest;
use deno_core::error::type_error;
use deno_core::error::AnyError;
use deno_core::op;
use deno_core::OpState;
use deno_core::ResourceId;
use deno_core::ZeroCopyBuf;
use rand::Rng;
use std::rc::Rc;

#[op(fast)]
pub fn op_node_create_hash(state: &mut OpState, algorithm: &str) -> u32 {
  state
    .resource_table
    .add(match digest::Context::new(algorithm) {
      Ok(context) => context,
      Err(_) => return 0,
    })
}

#[op(fast)]
pub fn op_node_hash_update(state: &mut OpState, rid: u32, data: &[u8]) -> bool {
  let context = match state.resource_table.get::<digest::Context>(rid) {
    Ok(context) => context,
    _ => return false,
  };
  context.update(data);
  true
}

#[op(fast)]
pub fn op_node_hash_update_str(
  state: &mut OpState,
  rid: u32,
  data: &str,
) -> bool {
  let context = match state.resource_table.get::<digest::Context>(rid) {
    Ok(context) => context,
    _ => return false,
  };
  context.update(data.as_bytes());
  true
}

#[op]
pub fn op_node_hash_digest(
  state: &mut OpState,
  rid: ResourceId,
) -> Result<ZeroCopyBuf, AnyError> {
  let context = state.resource_table.take::<digest::Context>(rid)?;
  let context = Rc::try_unwrap(context)
    .map_err(|_| type_error("Hash context is already in use"))?;
  Ok(context.digest()?.into())
}

#[op]
pub fn op_node_hash_digest_hex(
  state: &mut OpState,
  rid: ResourceId,
) -> Result<String, AnyError> {
  let context = state.resource_table.take::<digest::Context>(rid)?;
  let context = Rc::try_unwrap(context)
    .map_err(|_| type_error("Hash context is already in use"))?;
  let digest = context.digest()?;
  Ok(hex::encode(digest))
}

#[op]
pub fn op_node_generate_secret(buf: &mut [u8]) {
  rand::thread_rng().fill(buf);
}
