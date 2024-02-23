use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use cloud::rowacl::{AclType, RowAcl, RowAclChecker};
use dashmap::DashMap;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use crate::arena::App;
use crate::db::acl;
use crate::db::acl::acls;
use crate::db::app::{self, apps};

#[derive(Clone)]
pub struct Cache {
  db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
  pub apps_by_id: Arc<DashMap<String, App>>,
  pub acl_checker_by_app_id: Arc<DashMap<String, Arc<RowAclChecker>>>,
}

impl Cache {
  pub fn new(db_pool: Option<Pool<ConnectionManager<PgConnection>>>) -> Self {
    Self {
      db_pool,
      apps_by_id: Arc::new(DashMap::with_shard_amount(32)),
      acl_checker_by_app_id: Arc::new(DashMap::with_shard_amount(32)),
    }
  }

  pub async fn get_app(&self, app_id: &str) -> Result<Option<App>> {
    let app = self.apps_by_id.get(app_id).map(|w| w.value().clone());

    match app {
      Some(app) => Ok(Some(app)),
      None => self.fetch_and_cache_app(app_id).await,
    }
  }

  pub async fn get_app_acl_checker(
    &self,
    app_id: &str,
  ) -> Option<Arc<RowAclChecker>> {
    let acls = self
      .acl_checker_by_app_id
      .get(app_id)
      .map(|m| m.value().clone());

    match acls {
      Some(acls) => Some(acls),
      None => self.build_and_cache_app_acl_checker(app_id).await.ok(),
    }
  }

  async fn fetch_and_cache_app(&self, app_id: &str) -> Result<Option<App>> {
    let connection = &mut self
      .db_pool
      .clone()
      .ok_or(anyhow!("db pool not set"))?
      .get()?;

    let res = app::table
      .filter(apps::id.eq(app_id.to_string()))
      .filter(apps::archived_at.is_null())
      .first::<app::App>(connection);

    match res {
      Ok(db_app) => {
        let app = App {
          workspace_id: db_app.workspace_id.clone(),
          id: db_app.id.clone(),
          template: db_app.template.unwrap().try_into()?,
        };
        self.apps_by_id.insert(app.id.clone(), app.clone());
        Ok(Some(app))
      }
      Err(e) if e == diesel::NotFound => Ok(None),
      Err(e) => bail!("Failed to load app from db: {}", e),
    }
  }

  async fn build_and_cache_app_acl_checker(
    &self,
    app_id: &str,
  ) -> Result<Arc<RowAclChecker>> {
    let connection = &mut self
      .db_pool
      .clone()
      .ok_or(anyhow!("db pool not set"))?
      .get()?;

    let db_acls = acl::table
      .filter(acls::app_id.eq(Some(app_id)))
      .filter(acls::archived_at.is_null())
      .load::<acl::Acl>(connection)?;

    let acls = db_acls
      .into_iter()
      .filter_map(|acl| {
        let acl_types = match acl.access.to_uppercase().as_str() {
          "READ" => vec![AclType::Select],
          "WRITE" => vec![AclType::Insert],
          "UPDATE" => vec![AclType::Update],
          "DELETE" => vec![AclType::Delete],
          "OWNER" | "ADMIN" => vec![
            AclType::Select,
            AclType::Insert,
            AclType::Update,
            AclType::Delete,
          ],
          _ => vec![],
        };

        let metadata = acl.metadata;
        let table = metadata.get("table").and_then(|t| t.as_str());
        let filter = metadata.get("filter").and_then(|t| t.as_str());
        match (table, filter) {
          (Some(table), Some(filter)) => {
            let table = table.to_owned();
            let filter = filter.to_owned();
            Some(acl_types.into_iter().map(move |r#type| RowAcl {
              user_id: acl.user_id.to_owned(),
              table: table.clone(),
              r#type,
              filter: filter.clone(),
            }))
          }
          _ => None,
        }
      })
      .flatten()
      .collect::<Vec<RowAcl>>();

    let acl_checker = RowAclChecker::from(acls)?;
    self
      .acl_checker_by_app_id
      .insert(app_id.to_string(), acl_checker.into());

    self
      .acl_checker_by_app_id
      .get(app_id)
      .map(|checker| checker.value().clone())
      .ok_or(anyhow!("failed to get app acl checker"))
  }
}
