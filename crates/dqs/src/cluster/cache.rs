use crate::db::acl;
use crate::db::acl::acls;
use crate::db::app::{self, apps, App};
use anyhow::{anyhow, bail, Result};
use cloud::acl::{Access, Acl, AclEntity};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Cache {
  db_pool: Option<Pool<ConnectionManager<PgConnection>>>,
  /// app_id -> workspace_id
  pub workspace_apps: Arc<Mutex<HashMap<String, String>>>,
  pub acls: Arc<Mutex<HashMap<String, Box<Vec<Acl>>>>>,
}

impl Cache {
  pub fn new(db_pool: Option<Pool<ConnectionManager<PgConnection>>>) -> Self {
    Self {
      db_pool,
      workspace_apps: Arc::new(Mutex::new(HashMap::new())),
      acls: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  pub async fn get_workspace_id(&self, app_id: &str) -> Result<Option<String>> {
    let workspace_id = {
      let map = self.workspace_apps.lock().await;
      map.get(app_id).map(|w| w.to_string())
    };

    match workspace_id {
      Some(id) => Ok(Some(id)),
      None => self
        .fetch_and_cache_app(app_id)
        .await
        .map(|a| a.map(|a| a.workspace_id)),
    }
  }

  pub async fn get_workspace_acls(
    &self,
    workspace_id: &str,
  ) -> Option<Box<Vec<Acl>>> {
    let acls = {
      let map = self.acls.lock().await;
      map.get(workspace_id).map(|m| m.clone())
    };

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
      Ok(app) => {
        let mut map = self.workspace_apps.lock().await;
        map.insert(app.id.clone(), app.workspace_id.clone());
        Ok(Some(app))
      }
      Err(e) if e == diesel::NotFound => Ok(None),
      Err(e) => bail!("{}", e),
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
      .map(|acl| Acl {
        id: acl.id.to_owned(),
        user_id: acl.user_id.to_owned(),
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
      })
      .collect::<Vec<Acl>>();

    let mut map = self.acls.lock().await;
    map.insert(workspace_id.to_string(), acls.into());

    map
      .get(workspace_id)
      .map(|acls| acls.clone())
      .ok_or(anyhow!("failed to get workspace acls"))
  }
}
