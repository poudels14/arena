use std::sync::Arc;

use anyhow::Result;
use dashmap::DashMap;

pub mod handlers;
mod runtime;
mod runtime_spec;

use runtime::Runtime;
use runtime_spec::RuntimeImage;

#[derive(Clone)]
pub struct Cluster {
  runtimes: Arc<DashMap<String, Runtime>>,
}

impl Cluster {
  pub fn new() -> Self {
    Self {
      runtimes: Arc::new(DashMap::new()),
    }
  }

  pub async fn create_new_runtime(
    &self,
    _image: &RuntimeImage,
  ) -> Result<Runtime> {
    let runtime = runtime::init("/tmp/arena/python.sock").await?;
    // runtime.mount_fs().await?;
    self.runtimes.insert(runtime.id.clone(), runtime.clone());
    Ok(runtime)
  }

  pub fn get_runtime(&self, id: &str) -> Option<Runtime> {
    self.runtimes.get(id).map(|v| v.value().clone())
  }
}
