use anyhow::Result;
use deno_core::{
  op2, Extension, ExtensionFileSource, ExtensionFileSourceCode, Op, OpState,
  ToJsBuffer,
};
struct WasmerWasiBytes(&'static [u8; 327480]);

pub fn init() -> Extension {
  Extension {
    name: "arena/wasi",
    op_state_fn: Some(Box::new(move |state| {
      state.put::<WasmerWasiBytes>(WasmerWasiBytes(include_bytes!(
        "../../../../../../js/arena-runtime/libs/wasi/deno/pkg/wasmer_wasi_js_bg.wasm"
      )));
    })),
    ops: vec![op_read_wasmer_wasi_bytes::DECL].into(),
    js_files: vec![ExtensionFileSource {
      specifier: "setup",
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../../../js/arena-runtime/dist/wasmer-wasi.js"
      )),
    }]
    .into(),
    enabled: true,
    ..Default::default()
  }
}

#[op2]
#[serde]
pub fn op_read_wasmer_wasi_bytes(state: &mut OpState) -> Result<ToJsBuffer> {
  let bytes = state.borrow_mut::<WasmerWasiBytes>();
  Ok(bytes.0.to_vec().into())
}
