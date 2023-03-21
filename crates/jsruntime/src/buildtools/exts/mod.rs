use crate::config::ResolverConfig;
use deno_core::Extension;
use std::path::{Path, PathBuf};

pub mod resolver;
pub mod transpiler;

#[derive(Clone)]
pub(crate) struct BuildConfig {
  root: PathBuf,
  resolver: ResolverConfig,
}

pub fn init(
  project_root: &Path,
  resolver_config: ResolverConfig,
) -> Vec<Extension> {
  let root = project_root.to_path_buf();

  vec![
    Extension::builder("<arena/buildtools>")
      .state(move |state| {
        state.put::<BuildConfig>(BuildConfig {
          root: root.clone(),
          resolver: resolver_config.clone(),
        });
      })
      .build(),
    transpiler::init(),
    resolver::init(),
  ]
}
