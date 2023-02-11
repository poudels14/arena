mod fs;

pub use fs::FsModuleLoader;
pub use fs::ModuleLoaderOption;

use indexmap::IndexMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct ModuleLoaderConfig {
  /// The root directory of the project. It's usually where package.json is
  pub project_root: PathBuf,

  /// Module path alias as used by node resolvers, ViteJs, etc
  pub alias: Option<IndexMap<String, String>>,
}
