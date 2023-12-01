use std::sync::Arc;

use dashmap::DashMap;
use pgwire::api::portal::Portal;
use pgwire::api::stmt::StoredStatement;
use pgwire::api::store::PortalStore;

use super::statement::ArenaQuery;

#[allow(unused)]
pub struct ArenaPortalStore {
  portals: DashMap<String, Arc<Portal<ArenaQuery>>>,
  statements: DashMap<String, Arc<StoredStatement<ArenaQuery>>>,
}

impl ArenaPortalStore {
  pub fn new() -> Self {
    Self {
      portals: DashMap::new(),
      statements: DashMap::new(),
    }
  }
}

impl PortalStore for ArenaPortalStore {
  type Statement = ArenaQuery;

  fn put_portal(&self, _portal: Arc<Portal<Self::Statement>>) {
    // self.portals.insert(portal.name().to_string(), portal);
    unimplemented!()
  }

  fn get_portal(&self, _name: &str) -> Option<Arc<Portal<Self::Statement>>> {
    // self.portals.get(name).map(|kv| kv.value().clone());
    unimplemented!()
  }

  fn rm_portal(&self, _name: &str) {
    unimplemented!()
  }

  fn put_statement(&self, _statement: Arc<StoredStatement<Self::Statement>>) {
    // self.statements.insert(statement.id().clone(), statement);
    unimplemented!()
  }

  fn get_statement(
    &self,
    _name: &str,
  ) -> Option<Arc<StoredStatement<Self::Statement>>> {
    unimplemented!()
  }

  fn rm_statement(&self, _name: &str) {
    unimplemented!()
  }
}
