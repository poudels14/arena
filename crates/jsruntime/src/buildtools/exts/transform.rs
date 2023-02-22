use crate::permissions::PermissionsContainer;
use crate::utils::fs::resolve_from_cwd;
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
struct TransformOptions {
  /// disabled if not set
  /// only "inline" options supported right now
  source_map: Option<String>,
}

#[derive(Serialize)]
struct TransformResult {
  /// transpiled code
  code: Option<StringOrBuffer>,
}

pub fn init() -> Extension {
  Extension::builder("<arena/buildtools/transforms>")
    .ops(vec![
      op_buildtools_transform_sync::decl(),
      op_buildtools_transform_file_async::decl(),
    ])
    .js(vec![(
      "<arena/buildtools/transforms>",
      include_str!("./transform.js"),
    )])
    .build()
}

#[op]
async fn op_buildtools_transform_file_async(
  state: Rc<RefCell<OpState>>,
  filename: String,
  options: TransformOptions,
) -> Result<TransformResult> {
  let resolved_path = resolve_from_cwd(&Path::new(&filename))?;

  {
    let mut state = state.borrow_mut();
    let permissions = state.borrow_mut::<PermissionsContainer>();
    permissions.check_read(&resolved_path)?;
  }

  let code = tokio::fs::read_to_string(resolved_path).await?;
  transform_code(filename, &code, &options)
}

#[op]
fn op_buildtools_transform_sync(
  _state: &mut OpState,
  code: String,
  options: TransformOptions,
) -> Result<TransformResult> {
  transform_code("<code>".to_owned(), &code, &options)
}

fn transform_code(
  filename: String,
  code: &str,
  options: &TransformOptions,
) -> Result<TransformResult> {
  let parsed = deno_ast::parse_module(ParseParams {
    specifier: filename.to_owned(),
    text_info: SourceTextInfo::from_string(code.to_owned()),
    // Note(sagar): treat everything as typescript so that all transformations
    // are applied
    // TODO(sagar): allow configuring this with options argument
    media_type: MediaType::Tsx,
    capture_tokens: false,
    scope_analysis: false,
    maybe_syntax: None,
  })?;

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

  Ok(TransformResult {
    code: Some(StringOrBuffer::String(parsed_code)),
  })
}
