use arenasql::datafusion::LogicalPlan;
use arenasql::pgwire::api::results::FieldInfo;
use arenasql::pgwire::api::Type;
use getset::{Getters, Setters};

#[derive(Debug, Default, Clone, Getters, Setters)]
#[getset(get = "pub", set = "pub")]
pub struct ArenaPortalState {
  query_plan: Option<LogicalPlan>,
  /// List of parameter types for the query
  params: Option<Vec<Type>>,
  /// List of fields/columns in that the query plan returns
  fields: Vec<FieldInfo>,
}
