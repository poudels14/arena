use crate::WorkspaceServers;
use anyhow::Result;
use chrono::Duration;
use std::path::PathBuf;

#[derive(Clone)]
pub struct Config {
  /// Server port
  pub port: u16,

  /// The duration after which the live workspace will be shutdown
  pub ttl: Duration,

  /// The directory to load the workspaces to and serve from
  pub workspaces_dir: PathBuf,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Server {
  /// Server config
  config: Config,
  /// Workspace id -> LiveWorkspaces map
  workspace_servers: WorkspaceServers, // Arc<Mutex<HashMap<String, Arc<Mutex<LiveWorkspace>>>>>,
}

impl Server {
  pub fn new(config: Config) -> Self {
    Self {
      config,
      workspace_servers: WorkspaceServers::new(),
    }
  }

  pub async fn start(&self) -> Result<()> {
    // TODO(sagar)

    Ok(())
  }
}
