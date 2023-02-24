use super::BuildConfig;
use crate::config::ResolverConfig;
use crate::core::FsModuleResolver;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Result;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

impl Resource for FsModuleResolver {
  fn close(self: Rc<Self>) {}
}

pub fn init() -> Extension {
  Extension::builder("<arena/buildtools/resolver>")
    .ops(vec![op_resolver_new::decl(), op_resolver_resolve::decl()])
    .js(vec![(
      "<arena/buildtools/resolver>",
      include_str!("./resolver.js"),
    )])
    .build()
}

// TODO(sagar): should the resolver created here be dropped
// when resouce is closed?
#[op]
fn op_resolver_new(
  state: &mut OpState,
  config: Option<ResolverConfig>,
) -> Result<ResourceId> {
  let build_config = state.borrow_mut::<BuildConfig>();
  let resolver = FsModuleResolver::new(
    build_config.root.clone(),
    config.unwrap_or(build_config.resolver.clone()),
  );
  let rid = state.resource_table.add(resolver);
  Ok(rid)
}

#[op]
fn op_resolver_resolve(
  state: Rc<RefCell<OpState>>,
  rid: ResourceId,
  specifier: String,
  referrer: String,
) -> Result<String> {
  let state = state.borrow_mut();
  let resolver = state.resource_table.get::<FsModuleResolver>(rid)?;
  let build_config = state.borrow::<BuildConfig>();
  resolve(&resolver, &build_config.root, &referrer, &specifier)
}

pub(crate) fn resolve(
  resolver: &FsModuleResolver,
  root: &PathBuf,
  referrer: &str,
  specifier: &str,
) -> Result<String> {
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

  let resolved = resolver.resolve_import(&specifier, &referrer)?;
  let relative = pathdiff::diff_paths::<&PathBuf, &PathBuf>(
    &resolved.to_file_path().map_err(|e| anyhow!("{:?}", e))?,
    &root,
  );

  // Note(sagar): since all resolved paths are relative to project root,
  // prefix it with /
  Ok(format!("/{}", relative.unwrap().to_str().unwrap()))
}
