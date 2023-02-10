use deno_core::serde_v8;
use deno_core::v8;
use jsruntime::{IsolatedRuntime, RuntimeConfig};

#[tokio::main]
async fn main() {
  let r = IsolatedRuntime::new(RuntimeConfig {
    enable_console: true,
    ..Default::default()
  });

  let global_context;
  {
    let mut runtime = r.runtime.lock().unwrap();
    let scope = &mut runtime.handle_scope();

    let scope = &mut v8::EscapableHandleScope::new(scope);

    let context = v8::Context::new(scope);
    let global = context.global(scope);

    let deno_obj = v8::Object::new(scope);
    let deno_str = v8::String::new(scope, "ExampleObject").unwrap();
    global.set(scope, deno_str.into(), deno_obj.into());

    let core_obj = v8::Object::new(scope);
    let core_str = v8::String::new(scope, "core").unwrap();
    deno_obj.set(scope, core_str.into(), core_obj.into());

    let new_context = scope.escape(context);
    global_context = v8::Global::new(scope, context);

    // TODO(sagar): this isn't working. figure out how to make it work
    // using using deno/core/bindings.rs#L126 as reference
    // scope.set_default_context(new_context);
  }

  println!("getting another lock");
  let mut runtime = r.runtime.lock().unwrap();

  let result = runtime
    .execute_script(
      "<test>",
      "console.log(Object.keys(globalThis)); ExampleObject",
    )
    .unwrap();

  let isolate = runtime.v8_isolate();
  let mut scope = &mut v8::HandleScope::with_context(isolate, global_context);

  let local = v8::Local::new(&mut scope, result);
  let result = serde_v8::from_v8::<serde_json::Value>(&mut scope, local);
  println!("result = {:?}", result);
}
