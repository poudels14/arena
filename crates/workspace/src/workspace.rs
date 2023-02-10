use crate::registry::Registry;
use crate::ArenaConfig;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Workspace {
  /// The registry to load workspace from
  /// If none, only load workspace from local file system
  pub registry: Option<Registry>,

  /// The directory to load the workspace to
  pub dir: PathBuf,

  /// Arena config of the workspace
  pub config: ArenaConfig,
}

impl Workspace {
  /// Returns the entry file of the workspace
  pub fn entry_file(&self) -> PathBuf {
    self.dir.join(&self.config.server_entry)
  }
}
