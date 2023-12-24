use std::sync::Arc;

use arenasql::datafusion::LogicalPlan;
use dashmap::DashMap;
use getset::{Getters, Setters};
use pgwire::api::portal::Portal;
use pgwire::api::results::FieldInfo;
use pgwire::api::stmt::StoredStatement;
use pgwire::api::store::PortalStore;
use pgwire::api::Type;

use super::ArenaQuery;

#[derive(Debug, Default, Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct ArenaPortalState {
  query_plan: Option<LogicalPlan>,
  /// List of parameter types for the query
  params: Vec<Type>,
  /// List of fields/columns in that the query plan returns
  fields: Vec<FieldInfo>,
}

#[allow(unused)]
pub struct ArenaPortalStore {
  portals: DashMap<String, Arc<Portal<ArenaQuery, ArenaPortalState>>>,
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
  type State = ArenaPortalState;

  #[inline]
  fn put_portal(&self, portal: Arc<Portal<Self::Statement, Self::State>>) {
    self.portals.insert(portal.name().to_string(), portal);
  }

  #[inline]
  fn get_portal(
    &self,
    name: &str,
  ) -> Option<Arc<Portal<Self::Statement, Self::State>>> {
    self.portals.get(name).map(|kv| kv.value().clone())
  }

  #[inline]
  fn rm_portal(&self, _name: &str) {
    // TODO
    unimplemented!()
  }

  fn put_statement(&self, statement: Arc<StoredStatement<Self::Statement>>) {
    if statement.statement().stmts.len() > 1 {
      // TODO: patch pgwire to support returning Result here
      panic!("cannot insert multiple commands into a prepared statement")
    }
    self.statements.insert(statement.id().clone(), statement);
  }

  #[inline]
  fn get_statement(
    &self,
    name: &str,
  ) -> Option<Arc<StoredStatement<Self::Statement>>> {
    self.statements.get(name).map(|s| s.value().clone())
  }

  #[inline]
  fn rm_statement(&self, _name: &str) {
    // TODO
    unimplemented!()
  }
}
