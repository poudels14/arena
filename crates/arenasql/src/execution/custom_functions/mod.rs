use datafusion::execution::context::SessionContext as DfSessionContext;

mod vector;

pub use vector::L2_DISTANCE;

pub fn register_all(context: &DfSessionContext) {
  context.register_udf(L2_DISTANCE.clone());
}
