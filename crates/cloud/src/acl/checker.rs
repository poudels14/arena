use std::borrow::Cow;
use std::rc::Rc;

use anyhow::Result;
use runtime::deno::core::{op2, OpState, Resource, ResourceId};
use serde::{Deserialize, Serialize};

use super::{Access, AclEntity};
use crate::identity::Identity;

pub struct AclChecker {
  pub acls: Box<Vec<Acl>>,
}

impl Resource for AclChecker {
  fn name(&self) -> Cow<str> {
    "aclChecker".into()
  }

  fn close(self: Rc<Self>) {}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Acl {
  pub id: String,
  pub identity: Identity,
  pub workspace_id: String,
  pub entity: AclEntity,
  pub access: Access,
}

#[op2]
#[smi]
pub fn op_cloud_acl_new_checker(
  state: &mut OpState,
  #[serde] acls: Vec<Acl>,
) -> Result<ResourceId> {
  Ok(state.resource_table.add(AclChecker { acls: acls.into() }))
}

#[op2]
#[serde]
fn op_cloud_acl_filter_entity_by_access(
  state: &mut OpState,
  #[smi] checker_id: ResourceId,
  #[serde] identity: Identity,
  #[serde] access: Access,
  // workspace of the entities being filtered
  #[string] workspace_id: &str,
  #[serde] entities: Vec<AclEntity>,
) -> Result<Vec<AclEntity>> {
  let checker = state.resource_table.get::<AclChecker>(checker_id)?;
  Ok(
    filter_entity_by_access(
      &checker.acls,
      &identity,
      access,
      workspace_id,
      &entities,
    )?
    .iter()
    .map(|e| e.to_owned().to_owned())
    .collect::<Vec<AclEntity>>(),
  )
}

pub fn filter_entity_by_access<'a>(
  acls: &Vec<Acl>,
  identity: &Identity,
  access: Access,
  // workspace of the entities being filtered
  workspace_id: &str,
  entities: &'a Vec<AclEntity>,
) -> Result<Vec<&'a AclEntity>> {
  if entities.len() == 0 {
    return Ok(vec![]);
  }

  // TODO(sagar): write tests
  Ok(
    entities
      .iter()
      .filter(|e| {
        acls.iter().any(|acl| {
          // check if the user has workspace/entity level access
          (acl.workspace_id == workspace_id
            && acl.identity == *identity
            && acl.entity.comapre(e) >= 0
            && acl.access.compare(&access) >= 0)
            // Check entity level access
            || (acl.entity.comapre(e) >= 0
              && acl.access.compare(&access) >= 0
              // Note(sagar): if shared with another user, user_id will match;
              // if shared "publicly", allow access to everyone;
              && (acl.identity == Identity::Unknown || acl.identity == *identity))
        })
      })
      .collect::<Vec<&AclEntity>>(),
  )
}

pub fn has_entity_access(
  acls: &Vec<Acl>,
  identity: &Identity,
  access: Access,
  workspace_id: &str,
  entity: AclEntity,
) -> Result<bool> {
  filter_entity_by_access(acls, identity, access, workspace_id, &vec![entity])
    .map(|e| e.len() == 1)
}
