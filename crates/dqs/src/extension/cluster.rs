use crate::db;

use super::handle::DqsServerHandle;
use anyhow::Result;
use deno_core::{OpState, Resource, ResourceId};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::rc::Rc;
use tracing::debug;

#[derive(Clone, Debug)]
pub(crate) struct DqsCluster {
  internal: Rc<RefCell<Internal>>,
}

#[derive(Debug)]
pub(crate) struct Internal {
  pub instances: HashSet<ResourceId>,

  pub db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
}

impl Resource for DqsCluster {
  fn name(&self) -> Cow<str> {
    "dqsCluster".into()
  }

  fn close(self: Rc<Self>) {}
}

impl DqsCluster {
  pub fn new() -> Self {
    Self {
      internal: Rc::new(RefCell::new(Internal {
        instances: HashSet::new(),
        db_pool: None,
      })),
    }
  }

  pub fn get_db_pool(
    &mut self,
  ) -> Result<Pool<ConnectionManager<PgConnection>>> {
    let mut cluster = self.borrow_mut();
    match &cluster.db_pool {
      Some(pool) => Ok(pool.clone()),
      None => {
        let pool = db::create_connection_pool()?;
        cluster.db_pool = Some(pool.clone());
        Ok(pool)
      }
    }
  }

  pub fn add_instance(
    &mut self,
    state: &mut OpState,
    handle: DqsServerHandle,
  ) -> Result<ResourceId> {
    let handle_id = state.resource_table.add(handle);
    self.borrow_mut().instances.insert(handle_id);
    Ok(handle_id)
  }

  pub fn remove_instance(&mut self, handle_id: ResourceId) -> Result<()> {
    self.borrow_mut().instances.remove(&handle_id);
    debug!("removing DQS handle from DqsCluster");
    Ok(())
  }

  pub fn borrow<'a>(&'a self) -> Ref<'a, Internal> {
    self.internal.borrow()
  }

  pub fn borrow_mut<'a>(&'a mut self) -> RefMut<'a, Internal> {
    self.internal.borrow_mut()
  }
}
