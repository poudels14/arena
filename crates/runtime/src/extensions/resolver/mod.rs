use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op2;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;

use crate::config::node::ResolverConfig;
use crate::extensions::r#macro::js_dist;
use crate::extensions::BuiltinExtension;
use crate::resolver::FilePathResolver;
use crate::resolver::Resolver;

use super::SourceCode;

pub fn extension(root: PathBuf) -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::resolver::init_ops_and_esm(root)),
    vec![
      ("@arena/runtime/resolver", js_dist!("/resolver.js")),
      (
        "arena/resolver/setup",
        SourceCode::Runtime(include_str!("./resolver.js")),
      ),
    ],
  )
}

deno_core::extension!(
  resolver,
  ops = [op_resolver_new, op_resolver_resolve],
  options = { root: PathBuf },
  state = |state, options| {
    // TODO: remove me
    // let resolve = {
    //   let config = state.borrow::<RuntimeConfig>();
    //   config
    //     .resolve
    //     .clone()
    //     .unwrap_or(Default::default())
    // };
    state.put::<DefaultResolverConfig>(DefaultResolverConfig {
      root: options.root.to_owned(),
      config: Default::default(),
    });
  }
);

#[derive(Clone)]
pub struct DefaultResolverConfig {
  pub root: PathBuf,
  pub config: ResolverConfig,
}

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
  let default_config = state.borrow_mut::<DefaultResolverConfig>();
  let root = default_config.root.clone();
  let resolver = FilePathResolver::new(
    root.clone(),
    config.unwrap_or(default_config.config.clone()),
  );
  let rid = state.resource_table.add(resolver);
  Ok((
    rid,
    root
      .to_str()
      .map(|s| s.to_string())
      .ok_or(anyhow!("Failed to unwrap project root"))?,
  ))
}

#[op2]
#[string]
fn op_resolver_resolve(
  state: Rc<RefCell<OpState>>,
  #[smi] rid: ResourceId,
  #[string] specifier: String,
  #[string] referrer: String,
) -> Result<Option<String>> {
  let state = state.borrow_mut();
  let resolver = state.resource_table.get::<FilePathResolver>(rid)?;
  let default_config = state.borrow::<DefaultResolverConfig>();
  resolve(&resolver, &default_config.root, &referrer, &specifier)
}

pub(crate) fn resolve(
  resolver: &FilePathResolver,
  root: &PathBuf,
  referrer: &str,
  specifier: &str,
) -> Result<Option<String>> {
  // TODO(sagar): does this not check file access permission?
  let referrer = match referrer.starts_with(".") || referrer.starts_with("/") {
    true => {
      let p = root.join(referrer);
      format!("file://{}", p.to_str().unwrap())
    }
    false => bail!(
      "Only relative or absolute referrer is supported, passed = {:?}",
      &referrer
    ),
  };

  let resolved = resolver.resolve(&specifier, &referrer)?;
  let relative = pathdiff::diff_paths::<&PathBuf, &PathBuf>(
    &resolved.to_file_path().map_err(|e| anyhow!("{:?}", e))?,
    &root,
  );

  // Note(sagar): since all resolved paths are relative to project root,
  // prefix it with ./
  Ok(relative.and_then(|p| p.to_str().and_then(|s| Some(format!("./{s}")))))
}
