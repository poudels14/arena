mod babel;
pub mod jsx_analyzer;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use swc_ecma_visit::VisitWith;

pub use babel::BabelTranspiler;
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use url::Url;

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
  let mut jsx_analyzer = JsxAnalyzer::new();
  let module_filename = module_path.to_str().unwrap();
  let parsed = deno_ast::parse_program_with_post_process(
    ParseParams {
      specifier: module_filename.to_string(),
      text_info: SourceTextInfo::from_string(code.to_owned()),
      media_type: media_type.to_owned(),
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    },
    |program| {
      program.visit_children_with(&mut jsx_analyzer);
      program
    },
  )?;

  let transpiled_result = parsed.transpile(&EmitOptions {
    emit_metadata: true,
    inline_source_map: false,
    source_map: true,
    transform_jsx: jsx_analyzer.is_react,
    ..Default::default()
  })?;

  let transpiled_code = transpiled_result.text;

  if parsed.is_script() {
    let analysis = parsed.analyze_cjs();
    let exports_remap = analysis
      .exports
      .iter()
      .map(|export| format!("{} : module_export_{}", export, export))
      .collect::<Vec<String>>()
      .join(", ");

    let named_export = analysis
      .exports
      .iter()
      .map(|export| format!("module_export_{} as {}", export, export))
      .collect::<Vec<String>>()
      .join(", ");

    // TODO: collect reexports

    let module_dirname = module_path.parent().unwrap().to_str().unwrap();
    let module_fileurl = Url::from_file_path(module_path).unwrap();
    let module_fileurl = module_fileurl.as_str();

    return Ok(
      vec![
        &format!(
          "const require = __internalCreateRequire(\"{module_fileurl}\");"
        ),
        &format!("var __filename = \"{module_filename}\";"),
        &format!("var __dirname = \"{module_dirname}\";"),
        "var __commonJS = (cb, mod) => () =>",
        "\t(mod || cb((mod = { exports: {} }).exports, mod), mod.exports);",
        "let require_module = __commonJS((exports, module) => {{",
        &format!("{transpiled_code}"),
        "}});",
        "const named_exports_69 = require_module();",
        &format!("const {{ {exports_remap} }} = named_exports_69;"),
        &format!("export {{ {named_export} }};"),
        "export default named_exports_69;",
        &transpiled_result
          .source_map
          .map(|sm| {
            format!(
              "//# sourceMappingURL=data:application/json;base64,{}",
              base64::encode(sm)
            )
          })
          .unwrap_or_default(),
      ]
      .join("\n"),
    );
  }

  Ok(transpiled_code)
}
