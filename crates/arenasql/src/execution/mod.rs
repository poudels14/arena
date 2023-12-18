use std::sync::Arc;

mod config;
mod context;
mod custom_functions;
mod planner;
mod privilege;
mod transaction;

pub(crate) mod filter;
pub(crate) mod iterators;
pub(crate) mod response;

pub use config::{SessionConfig, TaskConfig};
pub use context::{SessionContext, DEFAULT_SCHEMA_NAME};
pub use privilege::Privilege;
pub use transaction::Transaction;

#[allow(dead_code)]
// re-export
pub type ExecutionPlan = Arc<dyn datafusion::physical_plan::ExecutionPlan>;
