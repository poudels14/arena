mod fs;

pub use fs::FsModuleLoader;
pub use fs::ModuleLoaderOption;

use crate::config::JsBuildConfig;
use std::path::PathBuf;

#[derive(Default, Debug)]
pub struct ModuleLoaderConfig {
  /// The root directory of the project. It's usually where package.json is
  pub project_root: PathBuf,

  pub(crate) build_config: JsBuildConfig,
}
