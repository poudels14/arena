use super::handle::DqsServerHandle;
use anyhow::Result;
use deno_core::{OpState, Resource, ResourceId};
use std::borrow::Cow;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashSet;
use std::rc::Rc;
use tracing::debug;

#[derive(Clone, Debug)]
pub(crate) struct DqsServers {
  internal: Rc<RefCell<Internal>>,
}

#[derive(Debug)]
pub(crate) struct Internal {
  pub instances: HashSet<ResourceId>,
}

impl Resource for DqsServers {
  fn name(&self) -> Cow<str> {
    "dqsServers".into()
  }

  fn close(self: Rc<Self>) {}
}

impl DqsServers {
  pub fn new() -> Self {
    Self {
      internal: Rc::new(RefCell::new(Internal {
        instances: HashSet::new(),
      })),
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
    debug!("removing DQS handle from DqsServers");
    Ok(())
  }

  pub fn borrow<'a>(&'a self) -> Ref<'a, Internal> {
    self.internal.borrow()
  }

  pub fn borrow_mut<'a>(&'a mut self) -> RefMut<'a, Internal> {
    self.internal.borrow_mut()
  }
}
