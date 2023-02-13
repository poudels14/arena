use anyhow::Result;
use deno_core::{op, Extension, OpState, ZeroCopyBuf};

struct WasmerWasiBytes(&'static [u8; 327480]);

pub fn init() -> Extension {
  Extension::builder("<arena/wasi>")
    .state(move |state| {
      state.put::<WasmerWasiBytes>(WasmerWasiBytes(include_bytes!(
        "../../../js/libs/src/wasi/deno/pkg/wasmer_wasi_js_bg.wasm"
      )));
      Ok(())
    })
    .ops(vec![op_read_wasmer_wasi_bytes::decl()])
    .js(vec![
      (
        "<arena/wasi/load>",
        include_str!("../../../js/libs/dist/wasmer-wasi.js"),
      ),
      (
        "<arena/wasi/setup>",
        r#"
      "use strict";
      ((global) => {
        if (!global.Arena) {
          global.Arena = {};
        };

        const {
          init,
          WASI,
          MemFS,
          JSVirtualFile,
          WasmerRuntimeError,
          InitGo,
        } = global.__bootstrap.wasi;

        const Go = InitGo(global);
        global.Arena.wasi = {
          init,
          WASI,
          MemFS,
          JSVirtualFile,
          WasmerRuntimeError,
          Go
        }
      })(globalThis);
      "#,
      ),
    ])
    .build()
}

#[op]
pub fn op_read_wasmer_wasi_bytes(state: &mut OpState) -> Result<ZeroCopyBuf> {
  let bytes = state.borrow_mut::<WasmerWasiBytes>();
  Ok(bytes.0.to_vec().into())
}
