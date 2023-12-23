use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use bytes::Bytes;
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use http::Method;
use swc_ecma_visit::FoldWith;
use swc_ecma_visit::VisitWith;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

use crate::extensions::server::response::ParsedHttpResponse;
use crate::extensions::server::HttpRequest;
use crate::extensions::transpiler::plugins;
use crate::extensions::transpiler::plugins::jsx_analyzer::JsxAnalyzer;

pub async fn transpile(
  transpiler_stream: mpsc::Sender<(
    HttpRequest,
    oneshot::Sender<ParsedHttpResponse>,
  )>,
  module_path: &Path,
  media_type: &MediaType,
  code: &str,
) -> Result<Arc<str>> {
  // TODO(sagar): strip out all dynamic transpiling for vms running deployed apps

  let mut jsx_analyzer = JsxAnalyzer::new();
  let parsed = deno_ast::parse_module_with_post_process(
    ParseParams {
      specifier: module_path.to_str().unwrap().to_owned(),
      text_info: SourceTextInfo::from_string(code.to_owned()),
      media_type: media_type.to_owned(),
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    },
    |p| {
      p.visit_children_with(&mut jsx_analyzer);
      p.fold_with(&mut plugins::commonjs::to_esm())
    },
  )?;

  let parsed_code = parsed
    .transpile(&EmitOptions {
      emit_metadata: true,
      transform_jsx: jsx_analyzer.is_react,
      ..Default::default()
    })?
    .text;

  let code = match module_path.extension() {
    Some(ext) if !jsx_analyzer.is_react && (ext == "tsx" || ext == "jsx") => {
      transpile_jsx(transpiler_stream, &parsed_code).await?
    }
    _ => parsed_code.to_owned(),
  };

  Ok(code.into())
}

async fn transpile_jsx<'a>(
  transpiler_stream: mpsc::Sender<(
    HttpRequest,
    oneshot::Sender<ParsedHttpResponse>,
  )>,
  code: &str,
) -> Result<String> {
  let (tx, rx) = oneshot::channel();
  transpiler_stream
    .send(((Method::POST, code).into(), tx))
    .await?;
  if let Ok(response) = rx.await {
    return response
      .data
      .and_then(|b| Some(Bytes::from(b).to_vec()))
      .and_then(|v| {
        Some(
          simdutf8::basic::from_utf8(&v)
            .expect("Error reading transpiled code")
            .to_owned(),
        )
      })
      .ok_or(anyhow!("Error reading transpiled code"));
  }
  bail!("Failed to transpile code using babel");
}
