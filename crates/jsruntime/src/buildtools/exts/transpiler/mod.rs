use crate::utils::fs::resolve_read_path;
use anyhow::Result;
use deno_ast::EmitOptions;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::StringOrBuffer;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

#[derive(Deserialize)]
struct TranspileOptions {
  /// disabled if not set
  /// only "inline" options supported right now
  source_map: Option<String>,
}

#[derive(Serialize)]
struct TranspileResult {
  /// transpiled code
  code: Option<StringOrBuffer>,
}

pub fn init() -> Extension {
  Extension::builder("<arena/buildtools/transpiler>")
    .ops(vec![
      op_transpiler_transpile_sync::decl(),
      op_transpiler_transpile_file_async::decl(),
    ])
    .js(vec![(
      "<arena/buildtools/transpiler>",
      include_str!("./transpiler.js"),
    )])
    .build()
}

#[op]
async fn op_transpiler_transpile_file_async(
  state: Rc<RefCell<OpState>>,
  filename: String,
  options: TranspileOptions,
) -> Result<TranspileResult> {
  let resolved_path = {
    let mut state = state.borrow_mut();
    resolve_read_path(&mut state, &Path::new(&filename))
  }?;

  let code = tokio::fs::read_to_string(resolved_path).await?;
  transpile_code(filename, &code, &options)
}

#[op]
fn op_transpiler_transpile_sync(
  _state: &mut OpState,
  code: String,
  options: TranspileOptions,
) -> Result<TranspileResult> {
  transpile_code("<code>".to_owned(), &code, &options)
}

fn transpile_code(
  filename: String,
  code: &str,
  options: &TranspileOptions,
) -> Result<TranspileResult> {
  let parsed = deno_ast::parse_module_with_post_process(
    ParseParams {
      specifier: filename.to_owned(),
      text_info: SourceTextInfo::from_string(code.to_owned()),
      // Note(sagar): treat everything as typescript so that all transformations
      // are applied
      // TODO(sagar): allow configuring this with options argument
      media_type: MediaType::Tsx,
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    },
    |p| p,
  )?;

  let parsed_code = parsed
    .transpile(
      // TODO(sagar): take all of these in options arg later
      &EmitOptions {
        emit_metadata: true,
        transform_jsx: false,
        inline_source_map: options
          .source_map
          .as_ref()
          .map(|m| m == "inline")
          .unwrap_or(false),
        ..Default::default()
      },
    )?
    .text;

  Ok(TranspileResult {
    code: Some(StringOrBuffer::String(parsed_code)),
  })
}
