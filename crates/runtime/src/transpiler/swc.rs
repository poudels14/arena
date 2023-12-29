use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use swc_ecma_visit::VisitWith;
use url::Url;

use crate::resolver::{ResolutionType, Resolver};
use crate::transpiler::jsx_analyzer::JsxAnalyzer;

#[derive(Clone)]
pub struct SwcTranspiler {
  resolver: Rc<dyn Resolver>,
}

impl SwcTranspiler {
  pub fn new(resolver: Rc<dyn Resolver>) -> Self {
    Self { resolver }
  }

  #[tracing::instrument(skip(self, code), level = "trace")]
  pub fn transpile(
    &self,
    module_path: &Path,
    media_type: &MediaType,
    code: &str,
    convert_cjs_to_esm: bool,
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

    if !convert_cjs_to_esm {
      return Ok(transpiled_result.text);
    }

    let transpiled_code = transpiled_result.text;
    if parsed.is_script() {
      let analysis = parsed.analyze_cjs();
      let mut exports = analysis.exports;
      analysis
        .reexports
        .iter()
        .map(|export| {
          get_cjs_reexports(&self.resolver, export.as_str(), &module_path)
        })
        .collect::<Result<Vec<Vec<String>>>>()?
        .into_iter()
        .flatten()
        .for_each(|export| {
          exports.push(export);
        });

      let exports_remap = exports
        .iter()
        .enumerate()
        .map(|(index, export)| {
          format!("{} : module_export_{}_{}", export, export, index)
        })
        .collect::<Vec<String>>()
        .join(", ");

      let named_export = exports
        .iter()
        .enumerate()
        .map(|(index, export)| {
          format!("module_export_{}_{} as {}", export, index, export)
        })
        .collect::<Vec<String>>()
        .join(", ");

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
}

fn get_cjs_reexports(
  resolver: &Rc<dyn Resolver>,
  specifier: &str,
  referrer: &Path,
) -> Result<Vec<String>> {
  let referrer_fileurl = Url::from_file_path(referrer).unwrap();
  let referrer_fileurl = referrer_fileurl.as_str();

  let resolved_path = resolver.resolve(
    specifier,
    referrer_fileurl,
    // Use Require resolution type since this is only called for cjs module
    ResolutionType::Require,
  )?;
  let specifier_filepath = resolved_path
    .to_file_path()
    .map_err(|e| anyhow!("Error converting Url to filepath{:?}", e))?;

  let code = std::fs::read_to_string(&specifier_filepath)?;
  let parsed = deno_ast::parse_program(ParseParams {
    specifier: specifier_filepath.to_str().unwrap().to_string(),
    text_info: SourceTextInfo::from_string(code),
    media_type: MediaType::Cjs,
    capture_tokens: false,
    scope_analysis: false,
    maybe_syntax: None,
  })?;

  if parsed.is_script() {
    let analysis = parsed.analyze_cjs();
    let reexports = analysis
      .reexports
      .iter()
      .map(|export| get_cjs_reexports(&resolver, export, &specifier_filepath))
      .collect::<Result<Vec<Vec<String>>>>()?
      .into_iter()
      .flatten()
      .collect();

    return Ok(vec![reexports, analysis.exports].concat());
  }
  Ok(vec![])
}
