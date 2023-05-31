use anyhow::{anyhow, bail, Result};
use common::deno::extensions::server::resonse::HttpResponse;
use common::deno::extensions::server::HttpRequest;
use common::deno::extensions::transpiler::plugins;
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use http::Method;
use http_body::Body;
use std::path::Path;
use std::sync::Arc;
use swc_ecma_visit::FoldWith;
use tokio::sync::mpsc;

pub fn transpile(
  transpiler_stream: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  module_path: &Path,
  media_type: &MediaType,
  code: &str,
) -> Result<Arc<str>> {
  // TODO(sagar): strip out all dynamic transpiling for vms running deployed apps

  let parsed = deno_ast::parse_module_with_post_process(
    ParseParams {
      specifier: module_path.to_str().unwrap().to_owned(),
      text_info: SourceTextInfo::from_string(code.to_owned()),
      media_type: media_type.to_owned(),
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    },
    |p| p.fold_with(&mut plugins::commonjs::to_esm()),
  )?;

  let parsed_code = parsed
    .transpile(&EmitOptions {
      emit_metadata: true,
      transform_jsx: false,
      ..Default::default()
    })?
    .text;

  let code = match module_path.extension() {
    Some(ext) if ext == "tsx" || ext == "jsx" => {
      transpile_jsx(transpiler_stream, &parsed_code)?
    }
    _ => parsed_code.to_owned(),
  };

  Ok(code.into())
}

fn transpile_jsx<'a>(
  transpiler_stream: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  code: &str,
) -> Result<String> {
  let transpiled_code: Result<String> = futures::executor::block_on(async {
    let (tx, mut rx) = mpsc::channel(2);
    transpiler_stream
      .send(((Method::POST, code).into(), tx))
      .await?;
    if let Some(mut response) = rx.recv().await {
      return Ok(
        std::str::from_utf8(&response.body_mut().data().await.unwrap()?)
          .map_err(|e| anyhow!("{}", e))?
          .to_owned(),
      );
    }
    bail!("Failed to transpile code using babel");
  });

  return transpiled_code;
}
