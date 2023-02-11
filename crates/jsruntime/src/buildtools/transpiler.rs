use super::analyzer;
use crate::IsolatedRuntime;
use anyhow::Result;
use serde_json::Value;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;

pub fn transpile(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  filename: &PathBuf,
  code: &[u8],
) -> Result<Box<[u8]>> {
  // TODO(sagar): strip out all dynamic transpiling for vms running deployed apps

  let mut analyzer = analyzer::Analyzer::new();
  let report = analyzer.analyze(
    &filename.to_string_lossy(),
    std::str::from_utf8(&code)?,
    &super::analyzer::Options {
      strip_typescript: true,
    },
  )?;

  let js_code = convert_to_string(&report)?;
  let code = if let Some(ext) = filename.extension() {
    let code = js_code.as_bytes().to_vec();
    if ext == "tsx" || ext == "jsx" {
      transpile_jsx(runtime, &code)?.as_bytes().to_vec()
    } else {
      // TODO(sagar): passing all code through babel for now but only transform
      // commonjs code to es6 if needed
      convert_to_es6(runtime, &code)?.as_bytes().to_vec()
    }
  } else {
    code.to_vec()
  };

  Ok(code.into_boxed_slice())
}

fn convert_to_string(report: &analyzer::Report) -> Result<String> {
  let mut buf = vec![];
  {
    let mut emitter = Emitter {
      cfg: swc_ecma_codegen::Config {
        minify: false,
        ..Default::default()
      },
      cm: report.source_map.clone(),
      comments: None,
      wr: JsWriter::new(report.source_map.clone(), "\n", &mut buf, None),
    };
    emitter.emit_module(&report.module).unwrap();
  }
  Ok(String::from_utf8(buf)?)
}

fn transpile_jsx(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  code: &[u8],
) -> Result<String> {
  execute_js(
    runtime,
    r#"
      ((code) => {
        const { babel, babelPlugins, babelPresets } = Arena;
        const { code : transpiledCode } = babel.transform(code, {
          presets: [
            // TODO(sagar): make this configurable to server/client
            [babelPresets.solid, { "generate": "ssr", "hydratable": true }]
          ],
          plugins: [
            [babelPlugins.transformCommonJs, { "exportsOnly": true }]
          ]
        });
        return transpiledCode;
      })
    "#,
    std::str::from_utf8(&code)?,
  )
}

fn convert_to_es6(
  runtime: Rc<RefCell<IsolatedRuntime>>,
  code: &[u8],
) -> Result<String> {
  execute_js(
    runtime,
    r#"
      ((code) => {
        const { babel, babelPlugins } = Arena;
        const { code : transpiledCode } = babel.transform(code, {
          plugins: [
            [babelPlugins.transformCommonJs, { "exportsOnly": true }]
          ]
        });
        return transpiledCode;
      })
    "#,
    std::str::from_utf8(&code)?,
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
