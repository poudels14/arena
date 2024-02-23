use std::sync::Arc;

use anyhow::{anyhow, Result};
use deno_core::{op2, OpState, ResourceId};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

mod checker;
pub use checker::{AclType, RowAcl, RowAclChecker};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RowAclOptions {
  acls: Vec<RowAcl>,
}

#[op2]
#[smi]
pub(crate) fn op_cloud_rowacl_new(
  state: &mut OpState,
  #[serde] options: RowAclOptions,
) -> Result<ResourceId> {
  let checker = RowAclChecker::from(options.acls)?;
  let id = state.resource_table.add(checker);
  Ok(id)
}

#[op2]
pub(crate) fn op_cloud_rowacl_has_access(
  state: &mut OpState,
  #[smi] id: ResourceId,
  #[string] user_id: String,
  #[string] table: String,
  #[serde] r#type: AclType,
) -> Result<bool> {
  let checker = state.resource_table.get::<RowAclChecker>(id)?;
  Ok(checker.has_query_access(&user_id, &table, r#type))
}

#[op2]
#[string]
pub(crate) fn op_cloud_rowacl_apply_filters(
  state: &mut OpState,
  #[smi] id: ResourceId,
  #[string] user_id: String,
  #[string] query: String,
) -> Result<String> {
  let checker = state.resource_table.get::<RowAclChecker>(id)?;
  checker.apply_sql_filter(&user_id, &query)
}

#[op2(fast)]
pub(crate) fn op_cloud_rowacl_close(
  state: &mut OpState,
  #[smi] id: ResourceId,
) -> Result<()> {
  let _ = state.resource_table.take::<RowAclChecker>(id);
  Ok(())
}

#[op2]
#[string]
pub(crate) fn op_cloud_default_rowacl_apply_filters(
  state: &mut OpState,
  #[string] user_id: String,
  #[string] query: String,
) -> Result<String> {
  let checker = state
    .try_borrow::<Arc<RwLock<RowAclChecker>>>()
    .ok_or(anyhow!("default RowAclChecker not found"))?;
  checker.read().apply_sql_filter(&user_id, &query)
}
