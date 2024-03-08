use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use arenasql_cluster::schema::ADMIN_USERNAME;
use cloud::identity::Identity;
use dqs::arena::{ArenaRuntimeState, MainModule};
use dqs::loaders::{FileTemplateLoader, Registry};
use dqs::runtime::deno::RuntimeOptions;
use runtime::deno::core::v8;
use runtime::deno::core::ModuleCode;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use runtime::permissions::PermissionsContainer;
use serde_json::Value;
use sqlx::migrate::MigrateDatabase;
use sqlx::Postgres;
use url::Url;

use crate::config::WorkspaceConfig;

#[derive(Debug)]
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
            registry: Registry {
              host: "n/a".to_owned(),
              api_key: "n/a".to_owned(),
            },
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
}
