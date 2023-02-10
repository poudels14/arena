use anyhow::{anyhow, bail, Result};
use arena_workspace::server::WorkspaceServerHandle;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct WorkspaceServers {
  handles: Arc<Mutex<HashMap<String, WorkspaceServerHandle>>>,
}

impl WorkspaceServers {
  pub fn new() -> Self {
    Self {
      handles: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub fn count(&self) -> Result<usize> {
    let handles = self.handles.lock().map_err(|e| anyhow!("{}", e))?;
    Ok(handles.len())
  }

  pub fn get(&self, _name: &str) -> Result<Arc<Mutex<WorkspaceServerHandle>>> {
    bail!("not implemented");
  }

  /// Wait for all workspace servers to shutdown
  pub async fn join(&self) -> Result<()> {
    // use tokio localset to wait for terminated signal from all servers
    // let local = task::LocalSet::new();
    bail!("not implemented");
  }
}
