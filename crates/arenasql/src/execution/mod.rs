use std::sync::Arc;

mod config;
mod context;
mod custom_functions;
mod planner;
mod transaction;

pub(crate) mod filter;
pub(crate) mod iterators;
pub mod response;

#[allow(unused_imports)]
pub use config::{SessionConfig, TaskConfig};
#[allow(unused_imports)]
pub use context::{SessionContext, DEFAULT_SCHEMA_NAME};
pub use transaction::Transaction;

#[allow(dead_code)]
// re-export
pub type ExecutionPlan = Arc<dyn datafusion::physical_plan::ExecutionPlan>;
