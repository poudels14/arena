mod workspace;

use std::sync::Arc;

use anyhow::Result;
use arenasql::execution::Privilege;
use arenasql_cluster::schema::{ClusterManifest, User, ADMIN_USERNAME};
use clap::Parser;
use dqs::cluster::{DqsCluster, DqsClusterOptions};
use dqs::db;
use runtime::deno::core::v8;
use tokio::sync::{broadcast, mpsc, oneshot};

use self::workspace::WorkspaceRouter;
use crate::config::WorkspaceConfig;
use crate::database::ArenasqlDatabase;
use crate::utils::templateloader::PortalTemplateLoader;
use crate::workspace::Workspace;

#[derive(Parser, Debug)]
pub struct Command {
  /// Server port
  #[arg(short, long, default_value_t = 4200)]
  pub port: u16,
}

impl Command {
  pub async fn execute(
    &self,
    v8_platform: v8::SharedRef<v8::Platform>,
    shutdown_signal: broadcast::Sender<()>,
  ) -> Result<()> {
    let workspace_config =
      WorkspaceConfig::load().expect("Error loading config");

    let workspace_database_password = workspace_config
      .workspace_db_password
      .clone()
      .unwrap_or_else(|| nanoid::nanoid!());

    let db_port = self
      .start_database(
        workspace_config.clone(),
        workspace_database_password.clone(),
        shutdown_signal.subscribe(),
      )
      .await?;

    let mut workspace = Workspace {
      config: workspace_config,
      db_port,
      port: self.port,
    };

    // if workspace db password isn't set, it probably means it's not setup yet
    let run_setup_script = workspace.config.workspace_db_password.is_none();

    if run_setup_script {
      workspace.config.workspace_db_password =
        Some(workspace_database_password);
      workspace.setup(v8_platform.clone()).await?;
      workspace.config.save().expect("Error saving portal config");
    }

    let workspace_database_url = workspace.database_url();
    std::env::set_var("DATABASE_URL", workspace_database_url.clone());
    std::env::set_var("JWT_SIGNING_SECRET", "portal-desktop-jwt-secret");

    let db_pool = db::create_connection_pool().await?;
    workspace.reset_database_cluster(&db_pool).await?;
    let dqs_cluster = DqsCluster::new(
      DqsClusterOptions {
        v8_platform: v8_platform.clone(),
        address: "127.0.0.1".to_owned(),
        port: self.port,
        dqs_egress_addr: None,
        template_loader: Arc::new(PortalTemplateLoader {}),
      },
      db_pool,
    )?;

    let (stream_tx, stream_rx) = mpsc::channel(10);
    let workspace_clone = workspace.clone();

    let (tx, rx) = oneshot::channel();
    rayon::spawn(move || {
      let rt = tokio::runtime::Builder::new_current_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("error setting up tokio runtime");

      let local = tokio::task::LocalSet::new();
      let _ = local.block_on(&rt, async {
        workspace::start_workspace_server(
          v8_platform,
          workspace_clone,
          stream_rx,
        )
        .await
        .expect("Error running workspace server");
      });
      let _ = tx.send(());
    });

    let shutdown_signal_rx = shutdown_signal.subscribe();
    let workspace_router = WorkspaceRouter::new(&workspace, stream_tx);

    dqs_cluster
      .start_server(
        Some(
          workspace_router
            .axum_router()
            .expect("creating workspace routes"),
        ),
        shutdown_signal_rx,
      )
      .await
      .unwrap();

    rx.await.unwrap();
    Ok(())
  }

  async fn start_database(
    &self,
    workspace_config: WorkspaceConfig,
    workspace_database_password: String,
    shutdown_signal: broadcast::Receiver<()>,
  ) -> Result<u16> {
    let (db_ready_tx, db_ready_rx) = oneshot::channel();
    let catalogs_dir = workspace_config.get_catalogs_dir();

    rayon::spawn(move || {
      let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .build()
        .unwrap();

      rt.block_on(async {
        let db = ArenasqlDatabase {};
        let manifest = ClusterManifest {
          users: vec![User {
            name: ADMIN_USERNAME.to_owned(),
            password: workspace_database_password,
            privilege: Privilege::SUPER_USER,
          }],
          catalogs_dir: catalogs_dir
            .to_str()
            .expect("Failed to get catalogs dir")
            .to_owned(),
          backup_dir: None,
          cache_size_mb: 10,
          checkpoint_dir: None,
          jwt_secret: None,
        };
        db.start(manifest, shutdown_signal, db_ready_tx)
          .await
          .unwrap();
      });
    });
    let port = db_ready_rx.await?;
    Ok(port)
  }
}
