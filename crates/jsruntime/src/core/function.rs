use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::error::JsError;
use deno_core::serde_v8;
use deno_core::v8;
use deno_core::JsRealm;
use deno_core::JsRuntime;
use serde_json::Value;
use smallvec::SmallVec;
use std::sync::{Arc, Mutex};

pub struct Function {
  runtime: Arc<Mutex<JsRuntime>>,
  cb: Option<v8::Global<v8::Function>>,
}

impl Function {
  /// Initializes a Javascript function
  /// If JsRealm is None, it uses global realm
  #[allow(dead_code)]
  pub(super) fn new(
    runtime: Arc<Mutex<JsRuntime>>,
    code: &str,
    realm: Option<JsRealm>,
  ) -> Result<Self> {
    let cb = {
      let mut runtime = runtime
        .lock()
        .map_err(|e| anyhow!("failed to get lock to runtime: {:?}", e))?;
      let global_relm = realm.unwrap_or(runtime.global_realm());
      let scope = &mut global_relm.handle_scope(runtime.v8_isolate());

      // TODO(sagar): need to drop this v8 global function when Function
      // is dropped
      deno_core::JsRuntime::eval::<v8::Function>(scope, code)
        .and_then(|cb| Some(v8::Global::new(scope, cb)))
    };

    Ok(Self { runtime, cb })
  }

  #[allow(dead_code)]
  pub fn execute(&self, args: Vec<Value>) -> Result<Option<Value>> {
    if self.cb.is_none() {
      return Ok(None);
    }

    let mut runtime = self
      .runtime
      .lock()
      .map_err(|e| anyhow!("failed to get lock to runtime: {:?}", e))?;
    let global_relm = runtime.global_realm();
    let scope = &mut global_relm.handle_scope(runtime.v8_isolate());

    let mut v8_args: SmallVec<[v8::Local<v8::Value>; 32]> =
      SmallVec::with_capacity(16 * 2);
    for arg in args {
      v8_args.push(serde_v8::to_v8(scope, arg).unwrap());
    }

    let tc_scope = &mut v8::TryCatch::new(scope);
    let js_recv = self.cb.as_ref().unwrap().open(tc_scope);

    let this = v8::undefined(tc_scope).into();
    let result = js_recv.call(tc_scope, this, v8_args.as_slice());

    if tc_scope.has_caught() {
      let exception = tc_scope.exception().unwrap();
      let js_error = JsError::from_v8_exception(tc_scope, exception);
      bail!("error: {:#}", js_error);
    }

    // TODO(sagar): handle errors

    match result {
      Some(v) => serde_v8::from_v8::<Value>(tc_scope, v)
        .map(|v| Some(v))
        .map_err(|e| anyhow!("{:?}", e)),
      None => Ok(None),
    }
  }
}
