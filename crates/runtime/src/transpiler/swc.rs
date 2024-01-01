use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceTextInfo};
use indexmap::IndexSet;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;
use url::Url;

use crate::extensions::resolver::inject_create_require;
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
      |mut program| {
        // Remove shebang!
        match &mut program {
          Program::Module(module) => {
            module.shebang = None;
          }
          Program::Script(script) => {
            script.shebang = None;
          }
        }
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

    let module_dirname = module_path.parent().unwrap().to_str().unwrap();
    let module_url = Url::from_file_path(module_path).unwrap();

    // Add this to all JS files so that they can use require("...")
    let header = vec![
      inject_create_require(&module_url),
      format!("var __filename = \"{module_filename}\";"),
      format!("var __dirname = \"{module_dirname}\";"),
    ]
    .join("\n");

    if parsed.is_script() {
      let analysis = parsed.analyze_cjs();
      let exports: IndexSet<String> = analysis
        .reexports
        .iter()
        .map(|export| {
          get_cjs_reexports(&self.resolver, export.as_str(), &module_path)
        })
        .collect::<Result<Vec<Vec<String>>>>()?
        .into_iter()
        .flatten()
        .chain(analysis.exports)
        .collect();

      let exports_remap = exports
        .iter()
        .enumerate()
        .map(|(index, export)| {
          format!("{} : m_export_{}_{}", export, export, index)
        })
        .collect::<Vec<String>>()
        .join(", ");

      let named_export = exports
        .iter()
        .enumerate()
        .map(|(index, export)| {
          format!("m_export_{}_{} as {}", export, index, export)
        })
        .collect::<Vec<String>>()
        .join(", ");

      // dont add default export if it's already there
      let default_export = if !exports.iter().any(|e| e == "default") {
        format!("export default named_exports_69;")
      } else {
        format!("")
      };

      return Ok(
        vec![
          &header,
          "var __commonJS = (cb, mod) => () =>",
          "\t(mod || cb((mod = { exports: {} }).exports, mod), mod.exports);",
          "let require_module = __commonJS((exports, module) => {{",
          &transpiled_result.text,
          "}});",
          "const named_exports_69 = require_module();",
          &format!("const {{ {exports_remap} }} = named_exports_69;"),
          &format!("export {{ {named_export} }};"),
          &default_export,
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

    Ok(vec![header, transpiled_result.text].join("\n"))
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
