use super::Workspace;
use crate::registry::Registry;
use crate::ArenaConfig;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
  /// The registry to load workspace from
  /// If none, only load workspace from local file system
  pub registry: Option<Registry>,

  /// The directory to load the workspace to
  pub dir: PathBuf,
}

/// Load a workspace for serving or editing
/// This will fetch files from Object storage if the files are not
/// already in the file system
///
/// All workspace related files need to be loaded to local file system for
/// editing or deployment.
/// In deployment more, this only loads essential files in the beginning and
/// lazy load other files as necessary
pub async fn load(config: Config) -> Result<Workspace> {
  let arena_config =
    ArenaConfig::from_path(&config.dir.join("arena.config.yaml"))?;
  Ok(Workspace {
    registry: config.registry.clone(),
    dir: config.dir.clone(),
    config: arena_config,
  })
}
