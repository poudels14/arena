use std::sync::Arc;

use arenasql::execution::{SessionContext, Transaction};
use arenasql::Result;
use dashmap::DashMap;
use derivative::Derivative;
use derive_builder::Builder;
use getset::Getters;
use parking_lot::Mutex;

#[derive(Builder, Derivative, Getters)]
#[derivative(Debug)]
pub struct AuthenticatedSession {
  #[getset(get = "pub")]
  id: String,
  #[getset(get = "pub")]
  user: String,
  #[getset(get = "pub")]
  database: String,
  context: SessionContext,
  #[derivative(Debug = "ignore")]
  #[builder(setter(strip_option), default)]
  active_transaction: Arc<Mutex<Option<Transaction>>>,
}

impl AuthenticatedSession {
  #[inline]
  pub fn get_active_transaction(&self) -> Option<Transaction> {
    self.active_transaction.lock().as_ref().map(|l| l.clone())
  }

  /// Creates a new transaction
  /// This is different than `begin_transaction` in that, the transaction
  /// returned from this won't be tracked and is meant to be used for
  /// unchained queries that don't have explicit `BEGIN/COMMIT`
  #[inline]
  pub fn create_transaction(&self) -> Result<Transaction> {
    self.context.begin_transaction()
  }

  /// Begins a new active transaction for this session
  #[inline]
  pub fn begin_new_transaction(&self) -> Result<Transaction> {
    let transaction = self.context.begin_transaction()?;
    let mut lock = self.active_transaction.lock();
    *lock = Some(transaction.clone());
    Ok(transaction)
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
