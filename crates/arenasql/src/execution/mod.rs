use std::sync::Arc;

mod config;
mod context;
mod custom_functions;
mod planner;
mod plans;
mod transaction;

pub(crate) mod filter;
pub(crate) mod iterators;
pub mod response;

#[allow(unused_imports)]
pub use config::{SessionConfig, TaskConfig};
pub use context::SessionContext;
pub use transaction::Transaction;

#[allow(dead_code)]
// re-export
pub type ExecutionPlan = Arc<dyn datafusion::physical_plan::ExecutionPlan>;
