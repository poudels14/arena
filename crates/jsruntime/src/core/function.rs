use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::error::JsError;
use deno_core::serde_v8;
use deno_core::v8;
use deno_core::JsRealm;
use deno_core::JsRuntime;
use futures::future::poll_fn;
use smallvec::SmallVec;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Function {
  runtime: Rc<RefCell<JsRuntime>>,
  cb: Option<v8::Global<v8::Function>>,
}

pub struct Value(Rc<RefCell<JsRuntime>>, v8::Global<v8::Value>);

impl Value {
  /// Returns serde_json::Value
  #[allow(dead_code)]
  pub fn get_value(&self) -> Result<serde_json::Value> {
    let mut runtime = self.0.borrow_mut();
    let scope = &mut runtime.handle_scope();

    let local_val = v8::Local::new(scope, &self.1);
    serde_v8::from_v8::<serde_json::Value>(scope, local_val)
      .map_err(|e| anyhow!("{:?}", e))
  }

  /// Returns serde_json::Value of a promise
  #[allow(dead_code)]
  pub async fn get_value_async(&self) -> Result<serde_json::Value> {
    let mut runtime = self.0.borrow_mut();

    let val = poll_fn(|cx| runtime.poll_value(&self.1, cx)).await.unwrap();
    let scope = &mut runtime.handle_scope();

    // TODO(sagar): is there a way to avoid changing local -> global -> local?
    let local_val = v8::Local::new(scope, val);
    serde_v8::from_v8::<serde_json::Value>(scope, local_val)
      .map_err(|e| anyhow!("{:?}", e))
  }
}

impl Function {
  /// Initializes a Javascript function
  /// If JsRealm is None, it uses global realm
  #[allow(dead_code)]
  pub(super) fn new(
    runtime: Rc<RefCell<JsRuntime>>,
    code: &str,
    realm: Option<JsRealm>,
  ) -> Result<Self> {
    let cb = {
      let mut runtime = runtime.borrow_mut();
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
  pub fn execute(&self, args: Vec<serde_json::Value>) -> Result<Option<Value>> {
    if self.cb.is_none() {
      return Ok(None);
    }

    let mut runtime = self.runtime.borrow_mut();
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
      // TODO(sagar): handle errors properly
      bail!("error: {:#}", js_error);
    }

    match result {
      Some(v) => Ok(Some(Value(
        self.runtime.clone(),
        // TODO(sagar): changing this to Global here and have to change it back to local
        // when getting the value. is there a way to avoid this?
        v8::Global::new(tc_scope, v),
      ))),
      None => Ok(None),
    }
  }
}
