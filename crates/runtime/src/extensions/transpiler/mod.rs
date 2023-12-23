pub mod plugins;

use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::Result;
use deno_ast::EmitOptions;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::{op2, Extension, Op, OpState, Resource, ResourceId};
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use swc_common::chain;
use swc_common::pass::Optional;
use swc_ecma_visit::FoldWith;
use swc_ecma_visit::VisitWith;

use self::plugins::jsx_analyzer::JsxAnalyzer;
use super::r#macro::js_dist;
use super::resolver::DefaultResolverConfig;
use crate::config::node::ResolverConfig;
use crate::extensions::BuiltinExtension;
use crate::permissions::resolve_read_path;
use crate::resolver::FilePathResolver;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init()),
    vec![("@arena/runtime/transpiler", js_dist!("/transpiler.js"))],
  )
}

pub fn init() -> Extension {
  Extension {
    name: "arena/buildtools/transpiler",
    ops: vec![
      op_transpiler_new::DECL,
      op_transpiler_transpile_sync::DECL,
      op_transpiler_transpile_file_async::DECL,
    ]
    .into(),
    enabled: true,
    ..Default::default()
  }
}

#[derive(Serialize)]
struct TranspileResult {
  /// transpiled code
  code: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct TranspilerConfig {
  /**
   * A set of key/value that will be replaced
   * when transpiling. Works similar to @rollup/plugin-replace
   */
  #[serde(default)]
  replace: IndexMap<String, String>,

  #[serde(default)]
  resolve_import: bool,

  #[serde(default)]
  resolver: Option<ResolverConfig>,

  #[serde(default)]
  source_map: Option<String>,
}

pub(crate) struct Transpiler {
  root: PathBuf,
  config: TranspilerConfig,
  resolver: Rc<FilePathResolver>,
}

impl Resource for Transpiler {
  fn close(self: Rc<Self>) {}
}

#[op2]
#[serde]
fn op_transpiler_new(
  state: &mut OpState,
  #[serde] config: TranspilerConfig,
) -> Result<(ResourceId, String)> {
  let build_config = state.borrow_mut::<DefaultResolverConfig>();

  let resolver_config = build_config
    .config
    .clone()
    .merge(config.resolver.clone().unwrap_or_default());

  let root = build_config.root.clone();
  let transpiler = Transpiler {
    root: root.clone(),
    config,
    resolver: Rc::new(FilePathResolver::new(root.clone(), resolver_config)),
  };

  let rid = state.resource_table.add(transpiler);
  Ok((
    rid,
    root
      .to_str()
      .map(|s| s.to_string())
      .ok_or(anyhow!("Failed to unwrap project root"))?,
  ))
}

/// Note(sagar): all filenames are resolved from root
#[op2(async)]
#[serde]
async fn op_transpiler_transpile_file_async(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] filename: String,
) -> Result<TranspileResult> {
  let (transpiler, resolved_path) = {
    let mut state = state.borrow_mut();
    (
      state.resource_table.get::<Transpiler>(rid)?,
      resolve_read_path(&mut state, &Path::new(&filename))?,
    )
  };

  let code = tokio::fs::read_to_string(&resolved_path).await?;
  transpile_code(transpiler, &resolved_path, &code)
}

#[op2]
#[serde]
fn op_transpiler_transpile_sync(
  state: &mut OpState,
  #[smi] rid: ResourceId,
  #[string] filename: String,
  #[string] code: String,
) -> Result<TranspileResult> {
  let transpiler = state.resource_table.get::<Transpiler>(rid)?;
  transpile_code(transpiler, &PathBuf::from(filename), &code)
}

fn transpile_code(
  transpiler: Rc<Transpiler>,
  filename: &PathBuf,
  code: &str,
) -> Result<TranspileResult> {
  let filename_str = filename.to_str().unwrap();

  let mut code = code.to_owned();
  let code = match transpiler.config.replace.is_empty() {
    true => code,
    false => {
      // TODO(sagar): optimize this?
      transpiler.config.replace.iter().for_each(|(key, value)| {
        code = code.replace(key, value);
      });
      code.to_owned()
    }
  };

  let mut jsx_analyzer = JsxAnalyzer::new();
  let parsed = deno_ast::parse_module_with_post_process(
    ParseParams {
      specifier: filename_str.to_string(),
      text_info: SourceTextInfo::from_string(code),
      // Note(sagar): treat everything as typescript so that all transformations
      // are applied
      // TODO(sagar): allow configuring this with options argument
      media_type: MediaType::Tsx,
      capture_tokens: false,
      scope_analysis: false,
      maybe_syntax: None,
    },
    |p| {
      p.visit_children_with(&mut jsx_analyzer);
      let config = &transpiler.as_ref().config;
      p.fold_with(&mut chain!(
        Optional::new(
          plugins::resolver::init(transpiler.clone(), filename_str),
          config.resolve_import,
        ),
        plugins::commonjs::to_esm(),
      ))
    },
  )?;

  let parsed_code = parsed
    .transpile(
      // TODO(sagar): take all of these in options arg later
      &EmitOptions {
        emit_metadata: true,
        transform_jsx: jsx_analyzer.is_react,
        inline_source_map: transpiler
          .config
          .source_map
          .as_ref()
          .map(|m| m == "inline")
          .unwrap_or(false),
        ..Default::default()
      },
    )?
    .text;
  Ok(TranspileResult { code: parsed_code })
}
