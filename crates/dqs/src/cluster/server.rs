use std::collections::HashMap;
use std::net::IpAddr;
use std::ops::Add;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, bail, Context, Result};
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::rowacl::RowAclChecker;
use common::beam;
use deno_core::v8;
use derivative::Derivative;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use runtime::extensions::server::response::ParsedHttpResponse;
use runtime::extensions::server::{HttpRequest, HttpServerConfig};
use runtime::permissions::PermissionsContainer;
use serde_json::Value;
use sqlx::types::chrono::Utc;
use sqlx::{Pool, Postgres};
use tokio::sync::{mpsc, oneshot, watch, Mutex};
use uuid::Uuid;

use crate::arena::{App, ArenaRuntimeState, MainModule};
use crate::config::workspace::WorkspaceConfig;
use crate::db::deployment::Deployment;
use crate::db::{self, database, deployment, resource};
use crate::jsruntime::RuntimeOptions;
use crate::jsruntime::{Command, ServerEvents};
use crate::loaders::moduleloader::AppkitModuleLoader;
use crate::loaders::TemplateLoader;

static RUNTIME_COUNTER: Lazy<Arc<AtomicUsize>> =
  Lazy::new(|| Arc::new(AtomicUsize::new(1)));

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct DqsServerOptions {
  pub id: String,
  /// App version or any other version
  pub version: String,
  pub workspace_id: String,
  pub root: Option<PathBuf>,
  pub module: MainModule,
  /// The IP address that DQS should use for outgoing network requests
  /// from DQS JS runtime
  pub dqs_egress_addr: Option<IpAddr>,

  #[derivative(Debug = "ignore")]
  pub template_loader: Arc<dyn TemplateLoader>,
  pub db_pool: Pool<Postgres>,
}

#[derive(Debug, Clone)]
pub enum DqsServerStatus {
  Starting(Arc<Mutex<bool>>),
  Ready(DqsServer),
}

#[derive(Debug, Clone)]
pub struct DqsServer {
  pub options: DqsServerOptions,
  pub http_channel:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  pub commands_channel: beam::Sender<Command, Value>,
  pub thread_handle: Arc<std::sync::Mutex<Option<JoinHandle<Result<()>>>>>,
}

impl DqsServer {
  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn spawn(
    v8_platform: v8::SharedRef<v8::Platform>,
    options: DqsServerOptions,
    exchange: Option<Exchange>,
    acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
  ) -> Result<(DqsServer, watch::Receiver<ServerEvents>)> {
    let (http_requests_tx, http_requests_rx) = mpsc::channel(200);
    let (events_tx, mut receiver) = watch::channel(ServerEvents::Init);
    let workspace_config = Self::load_workspace_config(&options).await?;
    let permissions = PermissionsContainer {
      net: workspace_config.runtime.net_permissions,
      ..Default::default()
    };

    let thread = thread::Builder::new().name(format!(
      "dqs-[{}]-{}",
      options.id,
      RUNTIME_COUNTER.fetch_add(1, Ordering::AcqRel)
    ));
    let options_clone = options.clone();

    let mut env_vars = match options.module.as_app() {
      Some(app) => Self::load_app_env_variables(
        &options.workspace_id,
        app,
        &options.db_pool,
      )
      .await
      .unwrap_or_default(),
      _ => HashMap::new(),
    };

    #[cfg(feature = "desktop")]
    env_vars.insert(
      "PORTAL_APP_HIDE_LOGS",
      EnvVar {
        id: "PORTAL_APP_HIDE_LOGS".to_owned(),
        key: "PORTAL_APP_HIDE_LOGS".to_owned(),
        value: Value::Bool(true),
        is_secret: false,
      },
    );

    let thread_handle = thread.spawn(move || {
      let state = ArenaRuntimeState {
        workspace_id: options.workspace_id.clone(),
        env_variables: EnvironmentVariableStore::new(env_vars),
        module: options.module.clone(),
      };

      let identity = match &state.module {
        MainModule::App { app } => Identity::App {
          id: app.id.clone(),
          owner_id: app.owner_id.clone(),
          system_originated: None,
        },
        MainModule::PluginWorkflowRun { workflow } => Identity::WorkflowRun {
          id: workflow.id.to_string(),
          system_originated: None,
        },
        _ => Identity::Unknown,
      };

      let jwt_secret = std::env::var("JWT_SIGNING_SECRET")?;
      let mut identity_json = serde_json::to_value(&identity)?;
      if let Value::Object(_) = identity_json {
        // TODO: make exp <5mins and figure out a way to update default headers
        identity_json["exp"] = Value::Number(
          SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .add(Duration::from_secs(60 * 60 * 24 * 30))
            .as_secs()
            .into(),
        );
      }
      let auth_header = jsonwebtoken::encode(
        &Header::new(Algorithm::HS512),
        &identity_json,
        &EncodingKey::from_secret((&jwt_secret).as_ref()),
      )
      .context("JWT encoding error")?;
      let egress_headers =
        Some(vec![("x-portal-authentication".to_owned(), auth_header)]);
      crate::jsruntime::start_runtime_server(
        RuntimeOptions {
          id: options.id,
          v8_platform,
          server_config: Some(HttpServerConfig::Stream(Arc::new(
            std::sync::Mutex::new(Some(http_requests_rx)),
          ))),
          egress_address: options.dqs_egress_addr,
          egress_headers,
          heap_limits: workspace_config
            .runtime
            .heap_limit_mb
            .map(|limit| (10 * 1024 * 1024, limit * 1024 * 1204)),
          permissions,
          exchange,
          acl_checker,
          state: Some(state),
          identity,
          module_loader: Some(Rc::new(AppkitModuleLoader {
            workspace_id: options.workspace_id,
            module: options.module,
            template_loader: options.template_loader,
          })),
        },
        events_tx,
      )
    })?;

    loop {
      if receiver.changed().await.is_err() {
        bail!("Events stream closed");
      }
      let event = receiver.borrow().clone();
      match event {
        ServerEvents::Init => {}
        ServerEvents::Started(commands) => {
          return Ok((
            DqsServer {
              options: options_clone,
              http_channel: http_requests_tx,
              commands_channel: commands,
              thread_handle: Arc::new(std::sync::Mutex::new(Some(
                thread_handle,
              ))),
            },
            receiver,
          ));
        }
        ServerEvents::Terminated(result) => {
          bail!("{:?}", &result)
        }
      }
    }
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn healthy(&self) -> Result<()> {
    let (tx, rx) = oneshot::channel::<ParsedHttpResponse>();
    let _ = self
      .http_channel
      .send((
        HttpRequest {
          method: "GET".to_owned(),
          url: format!("http://0.0.0.0/_admin/healthy"),
          headers: vec![],
          body: None,
        },
        tx,
      ))
      .await;
    let _ = rx.await;
    Ok(())
  }

  #[tracing::instrument(skip(self), level = "trace")]
  #[inline]
  pub async fn get_server_deployment(
    &self,
    id: &str,
  ) -> Result<Option<Deployment>> {
    deployment::get_deployment_with_id(&self.options.db_pool, id).await
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub async fn update_server_deployment(&self, node_id: &str) -> Result<()> {
    let app = self.options.module.as_app();
    let deployment = Deployment {
      id: self.options.id.clone(),
      node_id: node_id.to_string(),
      workspace_id: self.options.workspace_id.clone(),
      app_id: app.map(|a| a.id.clone()),
      app_template_id: app.map(|a| a.template.id.clone()),
      started_at: Utc::now().naive_utc(),
      last_heartbeat_at: None,
      reboot_triggered_at: None,
    };

    // Since arenasql doesn't support ON CONFLICT, delete existing deployment
    // first
    deployment::delete_deployment_with_id(
      &self.options.db_pool,
      &deployment.id,
    )
    .await?;
    deployment::insert_deployment(&self.options.db_pool, &deployment).await?;
    Ok(())
  }

  #[tracing::instrument(skip_all, err, level = "debug")]
  async fn load_workspace_config(
    options: &DqsServerOptions,
  ) -> Result<WorkspaceConfig> {
    let app = options.module.as_app();
    if app.is_none() {
      return Ok(WorkspaceConfig::default());
    }

    let workspace: db::workspace::Workspace = sqlx::query_as(
      "SELECT * FROM workspaces WHERE id = $1 AND archived_at IS NULL",
    )
    .bind(&options.workspace_id)
    .fetch_one(&options.db_pool)
    .await?;

    serde_json::from_value::<WorkspaceConfig>(workspace.config)
      .map_err(|e| anyhow!("{}", e))
  }

  #[tracing::instrument(skip_all, err, level = "debug")]
  async fn load_app_env_variables(
    workspace_id: &str,
    app: &App,
    pool: &Pool<Postgres>,
  ) -> Result<HashMap<String, EnvVar>> {
    let env_vars: Vec<resource::EnvVar> = sqlx::query_as(
      r#"SELECT * FROM environment_variables
    WHERE
      (
        (workspace_id = $1 AND app_id IS NULL AND app_template_id IS NULL) OR
        (app_id = $2) OR
        (app_template_id = $3 AND app_id IS NULL AND workspace_id IS NULL) OR
        (app_template_id IS NULL AND app_id IS NULL AND workspace_id IS NULL)
      ) AND archived_at IS NULL;
    "#,
    )
    .bind(workspace_id)
    .bind(&app.id)
    .bind(&app.template.id)
    .fetch_all(pool)
    .await
    .map_err(|e| {
      tracing::error!("{:?}", e);
    })
    .unwrap_or_default();

    let app_database =
      database::get_database_with_app_id(pool, workspace_id, &app.id)
        .await
        .map_err(|e| {
          tracing::error!("{:?}", e);
        })
        .unwrap_or_default();
    let database_cluster = match app_database {
      Some(ref db) if db.cluster_id.is_some() => {
        database::get_database_cluster_with_id(
          pool,
          db.cluster_id.as_ref().unwrap(),
        )
        .await
        .map_err(|e| {
          tracing::error!("{:?}", e);
        })
        .unwrap_or_default()
      }
      _ => None,
    };

    let mut resources = env_vars
      .iter()
      .map(|v| {
        (
          Uuid::new_v4().to_string(),
          EnvVar {
            id: v.id.clone(),
            key: v.key.clone(),
            value: Value::String(v.value.clone()),
            is_secret: false,
          },
        )
      })
      .collect::<HashMap<String, EnvVar>>();

    if let Some(ref db) = app_database {
      let db_name_id = Uuid::new_v4().to_string();
      resources.insert(
        db_name_id.clone(),
        EnvVar {
          id: db_name_id,
          key: "PORTAL_DATABASE_NAME".to_owned(),
          value: Value::String(db.id.clone()),
          is_secret: false,
        },
      );
      if let Some(user) = db.credentials.clone().unwrap_or_default().get("user")
      {
        let id = Uuid::new_v4().to_string();
        resources.insert(
          id.clone(),
          EnvVar {
            id,
            key: "PORTAL_DATABASE_USER".to_owned(),
            value: user.clone(),
            is_secret: false,
          },
        );
      }
      if let Some(password) =
        db.credentials.clone().unwrap_or_default().get("password")
      {
        let id = Uuid::new_v4().to_string();
        resources.insert(
          id.clone(),
          EnvVar {
            id,
            key: "PORTAL_DATABASE_PASSWORD".to_owned(),
            value: password.clone(),
            is_secret: false,
          },
        );
      }
    }
    if let Some(cluster) = database_cluster {
      let host_id = Uuid::new_v4().to_string();
      resources.insert(
        host_id.clone(),
        EnvVar {
          id: host_id,
          key: "PORTAL_DATABASE_HOST".to_owned(),
          value: Value::String(cluster.host.clone()),
          is_secret: false,
        },
      );

      let port_id = Uuid::new_v4().to_string();
      resources.insert(
        port_id.clone(),
        EnvVar {
          id: port_id,
          key: "PORTAL_DATABASE_PORT".to_owned(),
          value: Value::String(format!("{}", cluster.port)),
          is_secret: false,
        },
      );
    }

    Ok(resources)
  }
}
