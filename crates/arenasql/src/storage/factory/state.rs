use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use derive_builder::Builder;
use parking_lot::Mutex;
use tokio::sync::oneshot;

#[derive(Builder, Clone, Debug)]
pub struct StorageFactoryState {
  /// Total number of active transactions
  active_transactions_count: Arc<AtomicUsize>,
  /// This is set to true when graceful shutdown is calledP
  shutdown_triggered: Arc<AtomicBool>,
  /// If this is set to true, another transaction will load table
  /// schemas from store to get the updated copy. This is used to
  /// trigger reload when table schemas are updated
  schema_reload_triggered: Arc<AtomicBool>,
  shutdown_signal: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl Drop for StorageFactoryState {
  fn drop(&mut self) {
    if self.shutdown_triggered() && self.active_transactions() == 0 {
      if let Some(tx) = self.shutdown_signal.lock().take() {
        tx.send(()).unwrap();
      }
    }
  }
}

impl StorageFactoryState {
  #[inline]
  pub fn active_transactions(&self) -> usize {
    self.active_transactions_count.load(Ordering::Relaxed)
  }

  #[inline]
  pub fn increase_active_transaction_count(&self) {
    self
      .active_transactions_count
      .fetch_add(1, Ordering::AcqRel);
  }

  #[inline]
  pub fn reduce_active_transaction_count(&self) {
    self
      .active_transactions_count
      .fetch_sub(1, Ordering::AcqRel);
  }

  #[inline]
  pub fn should_reload_schema(&self) -> bool {
    self.schema_reload_triggered.load(Ordering::Acquire)
  }

  #[inline]
  pub fn reload_schema(&self) {
    self.schema_reload_triggered.store(true, Ordering::Release)
  }

  #[inline]
  pub fn trigger_shutdown(&self) {
    self.shutdown_triggered.store(true, Ordering::Release);
  }

  #[inline]
  pub fn shutdown_triggered(&self) -> bool {
    self.shutdown_triggered.load(Ordering::Acquire)
  }
}
