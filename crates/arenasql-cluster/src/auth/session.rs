use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use arenasql::{SessionContext, Transaction};
use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use parking_lot::Mutex;

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct AuthenticatedSession {
  pub id: u64,
  pub user: String,
  pub database: String,
  pub context: SessionContext,
  #[derivative(Debug = "ignore")]
  #[builder(setter(strip_option), default)]
  active_transaction: Arc<Mutex<Option<Transaction>>>,
}

impl AuthenticatedSession {
  #[inline]
  pub fn get_active_transaction(&self) -> Option<Transaction> {
    self.active_transaction.lock().as_ref().map(|l| l.clone())
  }

  #[inline]
  pub fn set_active_transaction(
    &self,
    transaction: Transaction,
  ) -> Option<Transaction> {
    let mut lock = self.active_transaction.lock();
    *lock = Some(transaction.clone());
    Some(transaction)
  }

  /// This removes the chained transaction associated with the session
  /// This should be called after the command COMMIT/ROLLBACK is explicitly
  /// called
  /// The session won't start another chained transaction until BEGIN is called
  /// again
  #[inline]
  pub fn clear_transaction(&self) {
    let mut lock = self.active_transaction.lock();
    *lock = None;
  }
}

pub struct AuthenticatedSessionStore {
  next_session_id: Arc<AtomicU64>,
  sessions: DashMap<u64, Arc<AuthenticatedSession>>,
}

#[allow(dead_code)]
impl AuthenticatedSessionStore {
  pub fn new() -> Self {
    Self {
      next_session_id: Arc::new(AtomicU64::new(1)),
      sessions: DashMap::new(),
    }
  }

  pub fn generate_session_id(&self) -> u64 {
    self.next_session_id.fetch_add(1, Ordering::SeqCst)
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

  pub fn get_session(
    &self,
    session_id: u64,
  ) -> Option<Arc<AuthenticatedSession>> {
    self.sessions.get(&session_id).map(|kv| kv.value().clone())
  }

  pub fn remove_session(
    &self,
    session_id: u64,
  ) -> Option<Arc<AuthenticatedSession>> {
    self
      .sessions
      .remove(&session_id)
      .map(|(_, session)| session)
  }
}
