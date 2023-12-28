mod babel;
pub mod commonjs;
pub mod jsx_analyzer;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use swc_ecma_visit::VisitWith;

pub use babel::BabelTranspiler;
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};

use self::jsx_analyzer::JsxAnalyzer;

#[async_trait]
pub trait ModuleTranspiler {
  async fn transpile<'a>(
    &'a self,
    path: &PathBuf,
    code: &str,
  ) -> Result<Arc<str>>;
}

pub fn transpile_js(
  module_path: &Path,
  media_type: &MediaType,
  code: &str,
) -> Result<String> {
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
    |mut module| {
      module.visit_children_with(&mut jsx_analyzer);
      commonjs::to_esm(code, &mut module, true);
      module
    },
  )?;

  let parsed_code = parsed
    .transpile(&EmitOptions {
      emit_metadata: true,
      transform_jsx: jsx_analyzer.is_react,
      ..Default::default()
    })?
    .text;
  Ok(parsed_code)
}
