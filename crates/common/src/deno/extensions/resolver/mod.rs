use crate::config::ArenaConfig;
use crate::deno;
use crate::deno::extensions::extension::BuiltinExtension;
use crate::deno::resolver;
use crate::deno::resolver::fs::FsModuleResolver;
use crate::resolve_from_root;
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

pub fn extension(root: PathBuf) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init(root)),
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

pub(crate) fn init(root: PathBuf) -> Extension {
  Extension::builder("arena/buildtools/resolver")
    .ops(vec![op_resolver_new::decl(), op_resolver_resolve::decl()])
    .state(move |state| {
      let resolver = {
        let config = state.borrow::<ArenaConfig>();
        config
          .javascript
          .as_ref()
          .and_then(|j| j.resolve.clone())
          .unwrap_or(Default::default())
      };

      state.put::<BuildConfig>(BuildConfig {
        root: root.to_owned(),
        resolver,
      });
    })
    .build()
}

#[derive(Clone)]
pub struct BuildConfig {
  pub root: PathBuf,
  pub resolver: deno::resolver::Config,
}

impl Resource for FsModuleResolver {
  fn close(self: Rc<Self>) {}
}

// TODO(sagar): should the resolver created here be dropped
// when resouce is closed?
#[op]
fn op_resolver_new(
  state: &mut OpState,
  config: Option<resolver::Config>,
) -> Result<(ResourceId, String)> {
  let build_config = state.borrow_mut::<BuildConfig>();
  let root = build_config.root.clone();
  let resolver = FsModuleResolver::new(
    root.clone(),
    config.unwrap_or(build_config.resolver.clone()),
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
  let build_config = state.borrow::<BuildConfig>();
  resolve(&resolver, &build_config.root, &referrer, &specifier)
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
