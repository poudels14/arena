use std::sync::Arc;

mod config;
mod context;
mod custom_functions;
mod execution_plan;
mod locks;
mod planner;
mod plans;
mod privilege;
mod state;
mod transaction;

pub(crate) mod filter;
pub(crate) mod iterators;
pub(crate) mod response;

pub mod factory;

pub use config::SessionConfig;
pub use context::{SessionContext, DEFAULT_SCHEMA_NAME};
pub use execution_plan::{
  CustomExecutionPlan, CustomExecutionPlanAdapter, ExecutionPlanExtension,
  ExecutionPlanResponse,
};
pub use locks::TableSchemaWriteLock;
pub use plans::{convert_literals_to_columnar_values, ScalarUdfExecutionPlan};
pub use privilege::Privilege;
pub use state::SessionState;
pub use transaction::{Transaction, TransactionHandle, TransactionLock};
pub mod tablescan {
  pub use super::iterators::HeapIterator;
}

#[allow(dead_code)]
// re-export
pub type ExecutionPlan = Arc<dyn datafusion::physical_plan::ExecutionPlan>;
