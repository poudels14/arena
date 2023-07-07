use crate::arena::ArenaConfig;
use crate::deno::extensions::BuiltinExtension;
use crate::deno::resolver::fs::FsModuleResolver;
use crate::node::ResolverConfig;
use crate::resolve_from_root;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

pub fn extension(root: PathBuf) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::resolver::init_ops_and_esm(root)),
    runtime_modules: vec![(
      "arena/resolver/setup",
      include_str!("./resolver.js"),
    )],
    snapshot_modules: vec![(
      "@arena/runtime/resolver",
      resolve_from_root!("../../js/arena-runtime/dist/resolver.js"),
    )],
  }
}

deno_core::extension!(
  resolver,
  ops = [op_resolver_new, op_resolver_resolve],
  options = { root: PathBuf },
  state = |state, options| {
    let resolve = {
      let config = state.borrow::<ArenaConfig>();
      config
        .javascript
        .as_ref()
        .and_then(|j| j.resolve.clone())
        .unwrap_or(Default::default())
    };
    state.put::<DefaultResolverConfig>(DefaultResolverConfig {
      root: options.root.to_owned(),
      config: resolve,
    });
  },
  customizer = |ext: &mut deno_core::ExtensionBuilder| {
    ext.force_op_registration();
  }
);

#[derive(Clone)]
pub struct DefaultResolverConfig {
  pub root: PathBuf,
  pub config: ResolverConfig,
}

impl Resource for FsModuleResolver {
  fn close(self: Rc<Self>) {}
}

// TODO(sagar): should the resolver created here be dropped
// when resouce is closed?
#[op]
fn op_resolver_new(
  state: &mut OpState,
  config: Option<ResolverConfig>,
) -> Result<(ResourceId, String)> {
  let default_config = state.borrow_mut::<DefaultResolverConfig>();
  let root = default_config.root.clone();
  let resolver = FsModuleResolver::new(
    root.clone(),
    config.unwrap_or(default_config.config.clone()),
    vec![],
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

#[op]
fn op_resolver_resolve(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  specifier: String,
  referrer: String,
) -> Result<Option<String>> {
  let state = state.borrow_mut();
  let resolver = state.resource_table.get::<FsModuleResolver>(rid)?;
  let default_config = state.borrow::<DefaultResolverConfig>();
  resolve(&resolver, &default_config.root, &referrer, &specifier)
}

pub(crate) fn resolve(
  resolver: &FsModuleResolver,
  root: &PathBuf,
  referrer: &str,
  specifier: &str,
) -> Result<Option<String>> {
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
