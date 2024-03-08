mod workspace;

use anyhow::Result;
use arenasql::execution::Privilege;
use arenasql_cluster::schema::{ClusterManifest, User, ADMIN_USERNAME};
use clap::Parser;
use common::required_env;
use dqs::cluster::{DqsCluster, DqsClusterOptions};
use dqs::db;
use dqs::loaders::Registry;
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::config::WorkspaceConfig;
use crate::database::ArenasqlDatabase;
use crate::workspace::Workspace;

use self::workspace::WorkspaceRouter;

#[derive(Parser, Debug)]
pub struct Command {
  /// Server port
  #[arg(short, long, default_value_t = 4200)]
  pub port: u16,
}

impl Command {
  pub async fn execute(
    &self,
    shutdown_signal: broadcast::Sender<()>,
  ) -> Result<()> {
    let registry_host = required_env!("REGISTRY_HOST");
    let registry_api_key = required_env!("REGISTRY_API_KEY");
    let _ = required_env!("JWT_SIGNING_SECRET");

    let workspace_config =
      WorkspaceConfig::load().expect("Error loading config");

    let shutdown_signal_rx = shutdown_signal.subscribe();
    let shutdown_signal_tx = shutdown_signal.clone();

    let (db_ready_tx, db_ready_rx) = oneshot::channel();

    let workspace_database_password = workspace_config
      .workspace_db_password
      .clone()
      .unwrap_or_else(|| nanoid::nanoid!());

    let workspace_database_password_clone = workspace_database_password.clone();
    let catalogs_dir = workspace_config.get_catalogs_dir();
    tokio::spawn(async move {
      let db = ArenasqlDatabase {};

      let manifest = ClusterManifest {
        users: vec![User {
          name: ADMIN_USERNAME.to_owned(),
          password: workspace_database_password_clone,
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
      let _ = db.start(manifest, shutdown_signal_rx, db_ready_tx).await;
      let _ = shutdown_signal_tx.send(());
    });

    let db_port = db_ready_rx.await?;
    let mut workspace = Workspace {
      config: workspace_config,
      db_port,
    };

    // if workspace db password isn't set, it probably means it's not setup yet
    let run_setup_script = workspace.config.workspace_db_password.is_none();

    if run_setup_script {
      workspace.config.workspace_db_password =
        Some(workspace_database_password);
      workspace.setup().await?;
      workspace.config.save().expect("Error saving portal config");
    }

    let workspace_database_url = workspace.database_url();
    std::env::set_var("DATABASE_URL", workspace_database_url.clone());

    let db_pool = db::create_connection_pool().await?;
    let dqs_cluster = DqsCluster::new(
      DqsClusterOptions {
        address: "127.0.0.1".to_owned(),
        port: self.port,
        dqs_egress_addr: None,
        registry: Registry {
          host: registry_host,
          api_key: registry_api_key,
        },
      },
      db_pool,
    )?;

    let (stream_tx, stream_rx) = mpsc::channel(10);
    tokio::spawn(async {
      rayon::scope(|_| {
        let rt = tokio::runtime::Builder::new_current_thread()
          .worker_threads(2)
          .enable_all()
          .build()
          .expect("error setting up tokio runtime");

        let local = tokio::task::LocalSet::new();
        let _ = local.block_on(&rt, async {
          workspace::start_workspace_server(workspace, stream_rx)
            .await
            .expect("Error running workspace server");
        });
      });
    });

    let shutdown_signal_rx = shutdown_signal.subscribe();
    tokio::spawn(async move {
      let workspace_router = WorkspaceRouter::new(stream_tx);
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
    });
    Ok(())
  }
}
