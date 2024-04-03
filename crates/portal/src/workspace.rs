use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;
use arenasql::chrono::{DateTime, Utc};
use arenasql_cluster::schema::ADMIN_USERNAME;
use cloud::identity::Identity;
use dqs::arena::{ArenaRuntimeState, MainModule};
use dqs::db::create_connection_pool;
use dqs::jsruntime::{self, RuntimeOptions};
use dqs::loaders::{AppkitModuleLoader, FileTemplateLoader};
use runtime::deno::core::v8;
use runtime::deno::core::ModuleCode;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use runtime::permissions::PermissionsContainer;
use serde_json::{json, Value};
use sqlx::migrate::MigrateDatabase;
use sqlx::{Pool, Postgres};
use url::Url;

use crate::config::WorkspaceConfig;
use crate::utils::assets::PortalAppModules;

#[derive(Debug, Clone)]
pub struct Workspace {
  pub config: WorkspaceConfig,
  pub port: u16,
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

  pub async fn setup(
    &self,
    v8_platform: v8::SharedRef<v8::Platform>,
  ) -> Result<()> {
    self.config.reset_files()?;
    self.create_portal_database().await?;
    self.run_workspace_db_migrations(v8_platform).await?;

    std::env::set_var("DATABASE_URL", self.database_url());
    let pool = create_connection_pool().await?;
    self.add_user(&pool).await?;
    self.add_default_app_templates(&pool).await?;
    self
      .trigger_tracking_event(
        "desktop-install",
        HashMap::from([
          ("version".to_owned(), env!("CARGO_PKG_VERSION").to_owned()),
          ("target".to_owned(), env!("TARGET").to_owned()),
        ]),
      )
      .await;
    Ok(())
  }

  async fn create_portal_database(&self) -> Result<()> {
    Postgres::create_database(&self.database_url()).await?;
    Ok(())
  }

  async fn run_workspace_db_migrations(
    &self,
    v8_platform: v8::SharedRef<v8::Platform>,
  ) -> Result<()> {
    let workspace_id = "workspace-desktop";
    let database_url = self.database_url();
    let _ = rayon::scope(|_| {
      let rt = tokio::runtime::Builder::new_current_thread()
        .thread_name("workspace-migration")
        .enable_io()
        .enable_time()
        .worker_threads(2)
        .build()?;

      let local = tokio::task::LocalSet::new();
      local.block_on(&rt, async {
        let mut runtime = jsruntime::new_runtime(RuntimeOptions {
          id: nanoid::nanoid!(),
          v8_platform,
          server_config: None,
          egress_address: None,
          egress_headers: None,
          heap_limits: None,
          permissions: PermissionsContainer::default(),
          exchange: None,
          acl_checker: None,
          state: Some(ArenaRuntimeState {
            workspace_id: workspace_id.to_owned(),
            module: MainModule::Inline {
              code: "".to_owned(),
            },
            env_variables: EnvironmentVariableStore::new(HashMap::from([(
              nanoid::nanoid!(),
              EnvVar {
                id: nanoid::nanoid!(),
                key: "DATABASE_URL".to_owned(),
                value: Value::String(database_url),
                is_secret: false,
              },
            )])),
          }),
          identity: Identity::Unknown,
          module_loader: Some(Rc::new(AppkitModuleLoader {
            workspace_id: workspace_id.to_owned(),
            module: MainModule::Inline {
              code: "".to_owned(),
            },
            template_loader: Arc::new(FileTemplateLoader {}),
          })),
        })
        .await?;

        let assets = PortalAppModules::new();
        let migration_script = assets
          .get_asset(&format!(
            "{}/{}/migrate.js",
            "workspace-desktop",
            env!("PORTAL_DESKTOP_WORKSPACE_VERSION")
          ))
          .expect("error loading migration script")
          .map(|bytes| std::str::from_utf8(bytes.as_ref()).unwrap().to_owned())
          .expect("error loading migration script");
        let mod_id = runtime
          .load_main_module(
            &Url::parse("file:///main.js").unwrap(),
            // TODO: encrypt JS code
            Some(ModuleCode::from(migration_script)),
          )
          .await?;

        let rx = runtime.mod_evaluate(mod_id);
        runtime.run_event_loop(Default::default()).await?;
        rx.await
      })
    });
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

    sqlx::query(
      r#"INSERT INTO environment_variables
    (id, name, key, value, created_at, updated_at)
    VALUES ($1, $2, $3, $4, $5, $5)
    "#,
    )
    .bind("1")
    .bind("Workspace Host")
    .bind("PORTAL_WORKSPACE_HOST")
    .bind(format!("http://localhost:{}/", self.port))
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

  pub async fn reset_database_cluster(
    &self,
    pool: &Pool<Postgres>,
  ) -> Result<()> {
    sqlx::query("DELETE FROM database_clusters WHERE 1=1")
      .execute(pool)
      .await?;

    sqlx::query(
      r#"INSERT INTO database_clusters
    (id, host, port, capacity, usage, credentials)
    VALUES ($1, $2, $3, $4, $5, $6)
    "#,
    )
    // Note: this has to be constant since dbs created for apps
    // will linked with the id of the cluster
    .bind("default-cluster")
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

  #[allow(unused)]
  pub async fn trigger_tracking_event(
    &self,
    event: &str,
    properties: HashMap<String, String>,
  ) {
    let now: DateTime<Utc> = SystemTime::now().into();
    let client = reqwest::Client::new();
    #[cfg(not(debug_assertions))]
    {
      let _ = client
        .post("https://app.posthog.com/capture/")
        .json(&json!({
            "api_key": "phc_hyT9GigBFrsv3HUkxDCiEettMlmdK4bT7M5SvczQ7fr",
            "event": event,
            "distinct_id": self.config.user_id,
            "properties": properties,
            "timestamp": now.to_rfc3339()
        }))
        .send()
        .await;
    }
  }
}
