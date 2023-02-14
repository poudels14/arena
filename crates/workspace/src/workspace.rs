use crate::registry::Registry;
use crate::WorkspaceConfig;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Workspace {
  /// The registry to load workspace from
  /// If none, only load workspace from local file system
  pub registry: Option<Registry>,

  /// The directory to load the workspace to
  pub dir: PathBuf,

  /// Arena config of the workspace
  pub config: WorkspaceConfig,

  /// Heap limit tuple: (initial size, max hard limit)
  pub heap_limits: Option<(usize, usize)>,
}

impl Workspace {
  pub fn project_root(&self) -> PathBuf {
    self.dir.clone()
  }

  /// Returns the entry file of the workspace
  pub fn entry_file(&self) -> PathBuf {
    self.dir.join(&self.config.server_entry)
  }
}
