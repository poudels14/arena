use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use cloud::acl::{Access, Acl, AclEntity};
use cloud::identity::Identity;
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
  pub acls: Arc<DashMap<String, Box<Vec<Acl>>>>,
}

impl Cache {
  pub fn new(db_pool: Option<Pool<ConnectionManager<PgConnection>>>) -> Self {
    Self {
      db_pool,
      apps_by_id: Arc::new(DashMap::with_shard_amount(32)),
      acls: Arc::new(DashMap::with_shard_amount(32)),
    }
  }

  pub async fn get_app(&self, app_id: &str) -> Result<Option<App>> {
    let app = self.apps_by_id.get(app_id).map(|w| w.value().clone());

    match app {
      Some(app) => Ok(Some(app)),
      None => self.fetch_and_cache_app(app_id).await,
    }
  }

  pub async fn get_workspace_acls(
    &self,
    workspace_id: &str,
  ) -> Option<Box<Vec<Acl>>> {
    let acls = self.acls.get(workspace_id).map(|m| m.value().clone());

    match acls {
      Some(acls) => Some(acls),
      None => self.fetch_and_cache_workspace_acls(workspace_id).await.ok(),
    }
  }

  async fn fetch_and_cache_app(&self, app_id: &str) -> Result<Option<App>> {
    let connection = &mut self
      .db_pool
      .clone()
      .ok_or(anyhow!("Db pool not set"))?
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

  async fn fetch_and_cache_workspace_acls(
    &self,
    workspace_id: &str,
  ) -> Result<Box<Vec<Acl>>> {
    let connection = &mut self
      .db_pool
      .clone()
      .ok_or(anyhow!("Db pool not set"))?
      .get()?;

    let db_acls = acl::table
      .filter(acls::workspace_id.eq(workspace_id.to_string()))
      .filter(acls::archived_at.is_null())
      .load::<acl::Acl>(connection)?;

    let acls = db_acls
      .iter()
      .map(|acl| {
        let identity = match acl.user_id.as_str() {
          "public" => Identity::Unknown,
          user_id => Identity::User {
            id: user_id.to_owned(),
          },
        };

        Acl {
          id: acl.id.to_owned(),
          identity,
          workspace_id: acl.workspace_id.to_owned(),
          access: Access::from(&acl.access),
          entity: match acl.app_id.as_ref() {
            Some(app_id) => AclEntity::App {
              id: app_id.to_owned(),
              path: acl.path.to_owned(),
            },
            None if acl.resource_id.is_some() => {
              AclEntity::Resource(acl.resource_id.to_owned().unwrap())
            }
            _ => AclEntity::Unknown,
          },
        }
      })
      .collect::<Vec<Acl>>();

    self.acls.insert(workspace_id.to_string(), acls.into());

    self
      .acls
      .get(workspace_id)
      .map(|acls| acls.value().clone())
      .ok_or(anyhow!("failed to get workspace acls"))
  }
}
