use common::config::ResolverConfig;
use deno_core::{Extension, ExtensionFileSource, ExtensionFileSourceCode};
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
    Extension::builder("arena/buildtools")
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

pub fn get_runtime_modules() -> Vec<ExtensionFileSource> {
  vec![
    // Note(sagar): this extension exposes built-in modules if buildtools
    // is enabled
    ExtensionFileSource {
      specifier: "setup".to_string(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "./buildtools.js"
      )),
    },
  ]
}
