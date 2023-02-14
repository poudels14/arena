use super::Workspace;
use crate::registry::Registry;
use crate::WorkspaceConfig;
use anyhow::Result;
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
/// In deployment more, this only loads essential files in the beginning and
/// lazy load other files as necessary
pub async fn load(options: Options) -> Result<Workspace> {
  let config =
    WorkspaceConfig::from_path(&options.dir.join("workspace.config.toml"))?;

  Ok(Workspace {
    registry: options.registry.clone(),
    dir: options.dir.clone(),
    config,
    heap_limits: options.heap_limits,
  })
}
