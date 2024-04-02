use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use cloud::rowacl::{AclType, RowAcl, RowAclChecker};
use dashmap::DashMap;
use parking_lot::RwLock;
use sqlx::Pool;
use sqlx::Postgres;

use crate::arena::App;
use crate::arena::Template;
use crate::db::acl;
use crate::db::app;

#[derive(Clone)]
pub struct Cache {
  db_pool: Option<Pool<Postgres>>,
  pub apps_by_id: Arc<DashMap<String, App>>,
  pub acl_checker_by_app_id: Arc<DashMap<String, Arc<RwLock<RowAclChecker>>>>,
}

impl Cache {
  pub fn new(db_pool: Option<Pool<Postgres>>) -> Self {
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
  ) -> Result<Arc<RwLock<RowAclChecker>>> {
    let checker = self
      .acl_checker_by_app_id
      .get(app_id)
      .map(|m| m.value().clone());

    match checker {
      Some(checker) => Ok(checker),
      None => {
        let acl_checker = Arc::new(RwLock::new(RowAclChecker::from(vec![])?));
        self
          .acl_checker_by_app_id
          .insert(app_id.to_string(), acl_checker.clone());
        let acls = self.fetch_app_acls(app_id).await?;

        acl_checker.write().set_acls(acls);
        Ok(acl_checker)
      }
    }
  }

  async fn fetch_and_cache_app(&self, app_id: &str) -> Result<Option<App>> {
    let pool = self.db_pool.clone().ok_or(anyhow!("db pool not set"))?;
    let res: Result<Option<app::App>, sqlx::Error> = sqlx::query_as(
      "SELECT * FROM apps WHERE id = $1 AND archived_at IS NULL",
    )
    .bind(app_id)
    .fetch_optional(&pool)
    .await;

    match res {
      Ok(db_app) => match db_app {
        Some(app) => {
          let app = App {
            workspace_id: app.workspace_id.clone(),
            id: app.id.clone(),
            template: Template {
              id: app.template_id.unwrap(),
              version: app.template_version.unwrap(),
            },
            owner_id: app.owner_id,
          };
          self.apps_by_id.insert(app.id.clone(), app.clone());
          Ok(Some(app))
        }
        None => Ok(None),
      },
      Err(e) => bail!("Failed to load app from db: {}", e),
    }
  }

  async fn fetch_app_acls(&self, app_id: &str) -> Result<Vec<RowAcl>> {
    let pool = self.db_pool.clone().ok_or(anyhow!("db pool not set"))?;
    let db_acls: Vec<acl::Acl> = sqlx::query_as(
      "SELECT * FROM acls WHERE app_id = $1 AND archived_at IS NULL",
    )
    .bind(app_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| anyhow!("loading acls {:?}", e))?;

    let acls = db_acls
      .into_iter()
      .filter_map(|acl| {
        let filters = acl
          .metadata
          .get("filters")
          .and_then(|t| t.as_array())
          .cloned();

        let user_id = acl.user_id.to_owned();
        match filters {
          Some(filters) => Some(
            filters
              .into_iter()
              .filter_map(move |filter| {
                let acl_command = match filter
                  .get("command")
                  .and_then(|c| c.as_str())?
                  .to_uppercase()
                  .as_str()
                {
                  "SELECT" => vec![AclType::Select],
                  "INSERT" => vec![AclType::Insert],
                  "UPDATE" => vec![AclType::Update],
                  "DELETE" => vec![AclType::Delete],
                  "*" => vec![
                    AclType::Select,
                    AclType::Insert,
                    AclType::Update,
                    AclType::Delete,
                  ],
                  _ => vec![],
                };

                let table =
                  filter.get("table").and_then(|t| t.as_str())?.to_owned();
                let condition =
                  filter.get("condition").and_then(|t| t.as_str())?.to_owned();
                Some(
                  acl_command
                    .iter()
                    .map(|r#type| RowAcl {
                      user_id: user_id.to_owned(),
                      table: table.to_owned(),
                      r#type: r#type.to_owned(),
                      filter: condition.to_owned(),
                    })
                    .collect::<Vec<RowAcl>>(),
                )
              })
              .flatten(),
          ),
          None => None,
        }
      })
      .flatten()
      .collect::<Vec<RowAcl>>();
    Ok(acls)
  }
}
