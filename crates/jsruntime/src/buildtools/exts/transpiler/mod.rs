mod plugins;

use super::BuildConfig;
use crate::config::ResolverConfig;
use crate::core::FsModuleResolver;
use crate::utils::fs::resolve_read_path;
use anyhow::anyhow;
use anyhow::Result;
use deno_ast::EmitOptions;
use deno_ast::MediaType;
use deno_ast::ParseParams;
use deno_ast::SourceTextInfo;
use deno_core::op;
use deno_core::Extension;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use deno_core::StringOrBuffer;
use indexmap::IndexMap;
use serde::Deserialize;
use serde::Serialize;
use std::cell::RefCell;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use swc_common::pass::Optional;
use swc_ecma_visit::FoldWith;

#[derive(Serialize)]
struct TranspileResult {
  /// transpiled code
  code: Option<StringOrBuffer>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
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
  resolver: Rc<FsModuleResolver>,
}

impl Resource for Transpiler {
  fn close(self: Rc<Self>) {}
}

pub fn init() -> Extension {
  Extension::builder("arena/buildtools/transpiler")
    .ops(vec![
      op_transpiler_new::decl(),
      op_transpiler_transpile_sync::decl(),
      op_transpiler_transpile_file_async::decl(),
    ])
    .js(vec![ExtensionFileSource {
      specifier: "setup".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "./transpiler.js"
      )),
    }])
    .build()
}

#[op]
fn op_transpiler_new(
  state: &mut OpState,
  config: TranspilerConfig,
) -> Result<(ResourceId, String)> {
  let build_config = state.borrow_mut::<BuildConfig>();

  let resolver_config = build_config
    .resolver
    .clone()
    .merge(config.resolver.clone().unwrap_or_default());

  let root = build_config.root.clone();
  let transpiler = Transpiler {
    root: root.clone(),
    config,
    resolver: Rc::new(FsModuleResolver::new(
      root.clone(),
      resolver_config,
      vec![],
    )),
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
#[op]
async fn op_transpiler_transpile_file_async(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  filename: String,
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

#[op]
fn op_transpiler_transpile_sync(
  state: &mut OpState,
  rid: ResourceId,
  code: String,
) -> Result<TranspileResult> {
  let transpiler = state.resource_table.get::<Transpiler>(rid)?;
  transpile_code(transpiler, &PathBuf::from("<code>"), &code)
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
      let config = &transpiler.as_ref().config;
      p.fold_with(&mut Optional::new(
        plugins::resolver::init(transpiler.clone(), filename_str),
        config.resolve_import,
      ))
    },
  )?;

  let parsed_code = parsed
    .transpile(
      // TODO(sagar): take all of these in options arg later
      &EmitOptions {
        emit_metadata: true,
        transform_jsx: false,
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

  Ok(TranspileResult {
    code: Some(StringOrBuffer::String(parsed_code)),
  })
}
