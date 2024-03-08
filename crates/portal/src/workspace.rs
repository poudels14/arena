use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use arenasql::chrono::Utc;
use arenasql_cluster::schema::ADMIN_USERNAME;
use cloud::identity::Identity;
use dqs::arena::{ArenaRuntimeState, MainModule};
use dqs::db::create_connection_pool;
use dqs::loaders::FileTemplateLoader;
use dqs::runtime::deno::RuntimeOptions;
use runtime::deno::core::v8;
use runtime::deno::core::ModuleCode;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use runtime::permissions::PermissionsContainer;
use serde_json::{json, Value};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Postgres};
use url::Url;

use crate::config::WorkspaceConfig;

#[derive(Debug, Clone)]
pub struct Workspace {
  pub config: WorkspaceConfig,
  pub db_port: u16,
}

impl Workspace {
  pub fn database_url(&self) -> String {
    format!(
      "postgresql://{}:{}@localhost:{}/portal",
      ADMIN_USERNAME,
      &self
        .config
        .workspace_db_password
        .as_ref()
        .expect("workspace_db_password must be set"),
      self.db_port
    )
  }

  pub async fn setup(&self) -> Result<()> {
    self.create_portal_database().await?;
    self.run_workspace_db_migrations().await?;

    std::env::set_var("DATABASE_URL", self.database_url());
    let pool = create_connection_pool().await?;
    self.add_user(&pool).await?;
    self.add_database_cluster(&pool).await?;
    self.add_default_app_templates(&pool).await?;
    Ok(())
  }

  async fn create_portal_database(&self) -> Result<()> {
    Postgres::create_database(&self.database_url()).await?;
    Ok(())
  }

  async fn run_workspace_db_migrations(&self) -> Result<()> {
    let v8_platform = v8::new_default_platform(0, false).make_shared();
    rayon::scope(|_| {
      let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .worker_threads(1)
        .build()?;

      let local = tokio::task::LocalSet::new();
      local.block_on(&rt, async {
        let mut runtime = dqs::runtime::deno::new(RuntimeOptions {
          id: nanoid::nanoid!(),
          db_pool: None,
          v8_platform,
          server_config: None,
          egress_address: None,
          egress_headers: None,
          heap_limits: None,
          permissions: PermissionsContainer::default(),
          exchange: None,
          acl_checker: None,
          state: ArenaRuntimeState {
            workspace_id: "workspace-1".to_owned(),
            module: MainModule::Inline {
              code: "".to_owned(),
            },
            env_variables: EnvironmentVariableStore::new(HashMap::from([(
              nanoid::nanoid!(),
              EnvVar {
                id: nanoid::nanoid!(),
                key: "DATABASE_URL".to_owned(),
                value: Value::String(self.database_url()),
                is_secret: false,
              },
            )])),
          },
          identity: Identity::Unknown,
          template_loader: Arc::new(FileTemplateLoader {}),
        })
        .await?;

        let mod_id = runtime
          .load_main_module(
            &Url::parse("file:///main.js").unwrap(),
            // TODO: encrypt JS code
            Some(ModuleCode::from_static(include_str!(
              "../../../js/workspace-desktop/dist/workspace/migrate.js"
            ))),
          )
          .await?;

        let rx = runtime.mod_evaluate(mod_id);
        runtime.run_event_loop(Default::default()).await?;
        rx.await
      })
    })?;

    Ok(())
  }

  async fn add_user(&self, pool: &Pool<Postgres>) -> Result<()> {
    sqlx::query(
      r#"INSERT INTO users
    (id, config, created_at)
    VALUES ($1, $2, $3)
    "#,
    )
    .bind(&self.config.user_id)
    .bind(json!({}))
    .bind(&Utc::now().naive_utc())
    .execute(pool)
    .await?;

    Ok(())
  }

  async fn add_default_app_templates(
    &self,
    pool: &Pool<Postgres>,
  ) -> Result<()> {
    sqlx::query(
      r#"INSERT INTO app_templates
    (id, name, default_version, owner_id, created_at)
    VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind("atlasai")
    .bind("Atlas AI")
    .bind(env!("PORTAL_DESKTOP_ATLAS_VERSION"))
    .bind(&self.config.user_id)
    .bind(&Utc::now().naive_utc())
    .execute(pool)
    .await?;

    sqlx::query(
      r#"INSERT INTO app_templates
    (id, name, default_version, owner_id, created_at)
    VALUES ($1, $2, $3, $4, $5)
    "#,
    )
    .bind("portal-drive")
    .bind("Drive")
    .bind(env!("PORTAL_DESKTOP_DRIVE_VERSION"))
    .bind(&self.config.user_id)
    .bind(&Utc::now().naive_utc())
    .execute(pool)
    .await?;

    Ok(())
  }

  async fn add_database_cluster(&self, pool: &Pool<Postgres>) -> Result<()> {
    sqlx::query(
      r#"INSERT INTO database_clusters
    (id, host, port, capacity, usage, credentials)
    VALUES ($1, $2, $3, $4, $5, $6)
    "#,
    )
    .bind(nanoid::nanoid!())
    .bind("localhost")
    .bind(self.db_port as i32)
    .bind(100)
    .bind(1)
    .bind(json!({
        "adminUser": ADMIN_USERNAME,
        "adminPassword": &self.config.workspace_db_password.as_ref().unwrap()
    }))
    .execute(pool)
    .await?;

    Ok(())
  }
}
