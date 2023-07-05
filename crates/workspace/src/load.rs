use super::Workspace;
use crate::registry::Registry;
use anyhow::{bail, Result};
use common::arena::ArenaConfig;
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Options {
  /// The registry to load workspace from
  /// If none, only load workspace from local file system
  pub registry: Option<Registry>,

  /// The directory to load the workspace to
  pub dir: PathBuf,

  /// Heap limit tuple: (initial size, max hard limit)
  pub heap_limits: Option<(usize, usize)>,
}

/// Load a workspace for serving or editing
/// This will fetch files from Object storage if the files are not
/// already in the file system
///
/// All workspace related files need to be loaded to local file system for
/// editing or deployment.
/// In deployment mode, this only loads essential files in the beginning and
/// lazy load other files as necessary
pub async fn load(options: Options) -> Result<Workspace> {
  if !options.dir.is_absolute() {
    bail!(
      "Workspace directory should be abolute path. current value = {:?}",
      options.dir
    );
  }

  let config = ArenaConfig::load(&options.dir)?;
  Ok(Workspace {
    registry: options.registry.clone(),
    dir: options.dir.clone(),
    config,
    heap_limits: options.heap_limits,
  })
}
