use datafusion::execution::context::SessionContext as DfSessionContext;

mod current_schema;
mod vector;

use current_schema::CURRENT_SCHEMA;
use vector::L2_DISTANCE;

pub fn register_all(context: &DfSessionContext) {
  context.register_udf(L2_DISTANCE.clone());
  context.register_udf(CURRENT_SCHEMA.clone());
}
