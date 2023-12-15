use std::sync::Arc;

use arenasql::{SessionContext, Transaction};
use dashmap::DashMap;
use derivative::Derivative;
use tokio::sync::Mutex;

use crate::error::ArenaClusterResult as Result;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct AuthenticatedSession {
  pub id: String,
  pub user: String,
  pub database: String,
  pub ctxt: SessionContext,
  #[derivative(Debug = "ignore")]
  pub transaction: Arc<Mutex<Arc<Transaction>>>,
}

impl AuthenticatedSession {
  #[inline]
  pub async fn get_transaction(&self) -> Result<Arc<Transaction>> {
    let txn = self.transaction.lock().await;
    Ok(txn.clone())
  }

  #[inline]
  pub async fn new_transaction(&self) -> Result<Arc<Transaction>> {
    let txn = Arc::new(self.ctxt.begin_transaction()?);
    let mut lock = self.transaction.lock().await;
    *lock = txn.clone();
    Ok(txn)
  }
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
