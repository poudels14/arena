use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op2;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use url::Url;

use super::SourceCode;
use crate::config::node::ResolverConfig;
use crate::config::RuntimeConfig;
use crate::extensions::r#macro::js_dist;
use crate::extensions::BuiltinExtension;
use crate::permissions;
use crate::resolver::FilePathResolver;
use crate::resolver::ResolutionType;
use crate::resolver::Resolver;

// Set default __internalCreateRequire that throws extension not enable error
static DEFAULT_CREATE_REQUIRE: &'static str = r#"
((global) => {
  global.__internalCreateRequire =
    global.__internalCreateRequire ||
    ((path) => {
      throw new Error("Resolver extension must be enabled to use require(...)");
    });
})(globalThis);"#;

pub fn inject_create_require(current_module: &Url) -> String {
  let module_url = current_module.as_str();
  format!(
    "{}\nconst require = __internalCreateRequire(\"{module_url}\");",
    DEFAULT_CREATE_REQUIRE
  )
}

pub fn extension(config: &ResolverConfig) -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::resolver::init_ops_and_esm(config.clone())),
    vec![
      ("@arena/runtime/resolver", js_dist!("/resolver.js")),
      (
        "arena/resolver/require",
        SourceCode::Preserved(include_str!("./require.js")),
      ),
    ],
  )
}

deno_core::extension!(
  resolver,
  ops = [
    op_resolver_new,
    op_resolver_resolve_alias,
    op_resolver_resolve,
    op_resolver_read_file,
  ],
  options = { config: ResolverConfig },
  state = move |state, options| {
    state.put::<ResolverConfig>(options.config.clone());
  }
);

impl Resource for FilePathResolver {
  fn close(self: Rc<Self>) {}
}

// TODO(sagar): should the resolver created here be dropped
// when resouce is closed?
#[op2]
#[serde]
fn op_resolver_new(
  state: &mut OpState,
  #[serde] config: Option<ResolverConfig>,
) -> Result<(ResourceId, String)> {
  let default_config = state.borrow_mut::<ResolverConfig>().clone();
  let resolver_config = match config {
    Some(config) => default_config.merge(config),
    None => default_config,
  };

  let runtime_config = state.borrow_mut::<RuntimeConfig>().clone();
  let root = runtime_config.project_root.clone();
  let resolver = FilePathResolver::new(root.clone(), resolver_config);
  let rid = state.resource_table.add(resolver);
  Ok((
    rid,
    root
      .to_str()
      .map(|s| s.to_string())
      .ok_or(anyhow!("Failed to unwrap project root"))?,
  ))
}

/// This will only resolve the alias if there's any
#[op2]
#[string]
fn op_resolver_resolve_alias(
  state: &mut OpState,
  #[string] specifier: &str,
) -> Result<Option<String>> {
  let config = state.borrow_mut::<ResolverConfig>().clone();
  Ok(config.alias.get(specifier).cloned())
}

/// Returns the resolved path relative to the project root and not
/// the referrer
#[tracing::instrument(skip(state, rid), level = "trace")]
#[op2]
#[string]
fn op_resolver_resolve(
  state: &mut OpState,
  #[smi] rid: ResourceId,
  #[string] specifier: String,
  #[string] referrer: String,
  #[serde] resolution_type: Option<ResolutionType>,
) -> Result<Option<String>> {
  let resolver = state.resource_table.get::<FilePathResolver>(rid)?;
  let runtime_config = state.borrow::<RuntimeConfig>();
  let resolved_path = resolve(
    resolver.as_ref(),
    &runtime_config.project_root,
    &referrer,
    &specifier,
    resolution_type.unwrap_or(ResolutionType::Import),
  )?;

  match resolved_path {
    Some(path) => {
      // Note: make sure the resolve path can be accessed
      // Just check the permission but return the above resolved path
      permissions::resolve_read_path(state, &Path::new(&path))?;
      Ok(Some(path))
    }
    None => Ok(None),
  }
}

#[tracing::instrument(skip(state), level = "trace")]
#[op2]
#[string]
fn op_resolver_read_file(
  state: &mut OpState,
  #[string] resolved_path: &str,
) -> Result<String> {
  let path = &Path::new(&resolved_path);
  permissions::check_read(state, &path)?;
  let content = std::fs::read_to_string(&path)?;
  // If it's a json file, prefix the content with "module.exports" to convert it
  // to JS
  if resolved_path.ends_with(".json") {
    return Ok(format!("module.exports = {}", content));
  }
  Ok(content)
}

pub(crate) fn resolve(
  resolver: &dyn Resolver,
  root: &PathBuf,
  referrer: &str,
  specifier: &str,
  resolution_type: ResolutionType,
) -> Result<Option<String>> {
  let referrer = match referrer.starts_with("file:///") {
    true => referrer.to_owned(),
    _ => match referrer.starts_with(".") || referrer.starts_with("/") {
      true => {
        let p = root.join(referrer);
        format!("file://{}", p.to_str().unwrap())
      }
      false => bail!(
        "Only relative or absolute referrer is supported, passed = {:?}",
        &referrer
      ),
    },
  };

  let resolved = resolver.resolve(&specifier, &referrer, resolution_type)?;
  let relative = pathdiff::diff_paths::<&PathBuf, &PathBuf>(
    &resolved.to_file_path().map_err(|e| anyhow!("{:?}", e))?,
    &root,
  );

  // Note(sagar): since all resolved paths are relative to project root,
  // prefix it with ./
  Ok(relative.and_then(|p| p.to_str().and_then(|s| Some(format!("./{s}")))))
}
