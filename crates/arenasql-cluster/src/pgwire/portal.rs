use arenasql::datafusion::LogicalPlan;
use getset::{Getters, Setters};
use pgwire::api::results::FieldInfo;
use pgwire::api::Type;

#[derive(Debug, Default, Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct ArenaPortalState {
  query_plan: Option<LogicalPlan>,
  /// List of parameter types for the query
  params: Vec<Type>,
  /// List of fields/columns in that the query plan returns
  fields: Vec<FieldInfo>,
}
