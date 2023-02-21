use anyhow::Result;
use deno_core::{op, Extension, OpState, ZeroCopyBuf};

struct WasmerWasiBytes(&'static [u8; 327480]);

pub fn init() -> Extension {
  Extension::builder("<arena/wasi>")
    .state(move |state| {
      state.put::<WasmerWasiBytes>(WasmerWasiBytes(include_bytes!(
        "../../../../../js/arena-runtime/libs/src/wasi/deno/pkg/wasmer_wasi_js_bg.wasm"
      )));
      Ok(())
    })
    .ops(vec![op_read_wasmer_wasi_bytes::decl()])
    .js(vec![(
      "<arena/wasi/load>",
      include_str!("../../../../../js/arena-runtime/libs/dist/wasmer-wasi.js"),
    )])
    .build()
}

#[op]
pub fn op_read_wasmer_wasi_bytes(state: &mut OpState) -> Result<ZeroCopyBuf> {
  let bytes = state.borrow_mut::<WasmerWasiBytes>();
  Ok(bytes.0.to_vec().into())
}
