use crate::IsolatedRuntime;
use anyhow::Result;
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use serde_json::Value;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

pub fn transpile(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  module_path: &Path,
  media_type: &MediaType,
  code: &str,
) -> Result<String> {
  // TODO(sagar): strip out all dynamic transpiling for vms running deployed apps

  let parsed = deno_ast::parse_module(ParseParams {
    specifier: module_path.to_str().unwrap().to_owned(),
    text_info: SourceTextInfo::from_string(code.to_owned()),
    media_type: media_type.to_owned(),
    capture_tokens: false,
    scope_analysis: false,
    maybe_syntax: None,
  })?;

  let parsed_code = parsed
    .transpile(&EmitOptions {
      emit_metadata: true,
      transform_jsx: false,
      ..Default::default()
    })?
    .text;

  let code = match module_path.extension() {
    Some(ext) if ext == "tsx" || ext == "jsx" => {
      transpile_jsx(runtime, &parsed_code)?
    }
    _ => parsed_code.to_owned(),
  };

  Ok(code)
}

fn transpile_jsx<'a>(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  code: &str,
) -> Result<String> {
  execute_js(
    runtime,
    r#"
      ((code) => {
        const { babel, babelPlugins, babelPresets } = Arena.BuildTools;
        const { code : transpiledCode } = babel.transform(code, {
          presets: [
            // Note(sagar): since the code transpiled here is only used in
            // server side, it should be transpiled for "ssr"
            [babelPresets.solidjs, {
              "generate": "ssr",
              "hydratable": String(Arena.env.ARENA_SSR) === "true"
            }]
          ],
          plugins: [
            [babelPlugins.transformCommonJs, { "exportsOnly": true }]
          ]
        });
        return transpiledCode;
      })
    "#,
    code,
  )
}

fn execute_js(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  code: &str,
  arg: &str,
) -> Result<String> {
  let mut runtime = runtime.borrow_mut();

  let function = runtime.init_js_function(code, None)?;
  let code = function
    .execute(vec![Value::String(arg.to_owned())])?
    .unwrap()
    .get_value()?;

  Ok(code.as_str().unwrap().to_owned())
}
