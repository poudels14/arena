use std::sync::Arc;

use arenasql::execution::SessionContext;
use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use getset::Getters;

#[derive(Builder, Derivative, Getters)]
#[derivative(Debug)]
pub struct AuthenticatedSession {
  #[getset(get = "pub")]
  id: String,
  #[getset(get = "pub")]
  user: String,
  #[getset(get = "pub")]
  database: String,
  #[getset(get = "pub")]
  context: SessionContext,
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

  #[tracing::instrument(skip(self), level = "trace")]
  pub fn put(
    &self,
    session: AuthenticatedSession,
  ) -> Arc<AuthenticatedSession> {
    let session = Arc::new(session);
    let old = self
      .sessions
      .insert(session.id.to_string(), session.clone());
    if old.is_some() {
      unreachable!("Session with same id already exists in the store")
    }
    session
  }

  pub fn get_session(
    &self,
    session_id: &str,
  ) -> Option<Arc<AuthenticatedSession>> {
    self.sessions.get(session_id).map(|kv| kv.value().clone())
  }

  #[tracing::instrument(skip(self), level = "trace")]
  pub fn remove_session(
    &self,
    session_id: &str,
  ) -> Option<Arc<AuthenticatedSession>> {
    self.sessions.remove(session_id).map(|(_, session)| session)
  }

  pub fn clear(&self) {
    self.sessions.clear()
  }
}
