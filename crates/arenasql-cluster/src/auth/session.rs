use std::sync::Arc;

use arenasql::{SessionContext, Transaction};
use dashmap::DashMap;
use derivative::Derivative;
use tokio::sync::Mutex;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct AuthenticatedSession {
  pub id: String,
  pub user: String,
  pub database: String,
  pub ctxt: SessionContext,
  #[derivative(Debug = "ignore")]
  pub transaction: Arc<Mutex<Option<Transaction>>>,
}

pub struct AuthenticatedSessionStore {
  sessions: DashMap<String, Arc<AuthenticatedSession>>,
}

#[allow(dead_code)]
impl AuthenticatedSessionStore {
  pub fn new() -> Self {
    Self {
      sessions: DashMap::new(),
    }
  }

  pub fn put(
    &self,
    session: AuthenticatedSession,
  ) -> Arc<AuthenticatedSession> {
    let session = Arc::new(session);
    let old = self.sessions.insert(session.id.clone(), session.clone());
    if old.is_some() {
      unreachable!("Session with same id already exists in the store")
    }
    session
  }

  pub fn get(&self, session_id: &str) -> Option<Arc<AuthenticatedSession>> {
    self.sessions.get(session_id).map(|kv| kv.value().clone())
  }

  pub fn remove(&self, session_id: &str) -> Option<Arc<AuthenticatedSession>> {
    self.sessions.remove(session_id).map(|(_, session)| session)
  }
}
