use crate::registry::Registry;
use anyhow::{anyhow, Result};
use common::arena::ArenaConfig;
use deno_core::normalize_path;
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

  /// Heap limit tuple: (initial size, max hard limit)
  pub heap_limits: Option<(usize, usize)>,
}

impl Workspace {
  pub fn project_root(&self) -> PathBuf {
    self.dir.clone()
  }

  /// Returns the entry file of the workspace
  pub fn server_entry(&self) -> Result<PathBuf> {
    let path = self.dir.join(&self.config.server.entry);

    path.canonicalize().map_err(|e| {
      anyhow!(
        "Error locating server entry {:?}. {}",
        normalize_path(path),
        e
      )
    })
  }
}
