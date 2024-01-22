use std::collections::HashMap;

use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use serde_json::Value;
use uuid::Uuid;

use super::app::App;
use super::MainModule;
use crate::db::database::{
  database_clusters, databases, Database, DatabaseCluster,
};
use crate::db::resource::{self, environment_variables};
use crate::loaders::registry::Registry;

#[derive(Debug, Clone)]
pub struct ArenaRuntimeState {
  pub workspace_id: String,
  pub registry: Registry,
  pub module: MainModule,
  pub env_variables: EnvironmentVariableStore,
}

impl ArenaRuntimeState {
  #[tracing::instrument(skip_all, err, level = "debug")]
  pub fn load_app_env_variables(
    workspace_id: &str,
    app: &App,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
  ) -> Result<EnvironmentVariableStore> {
    let query = resource::table
      .filter(
        environment_variables::workspace_id
          .eq(workspace_id.to_string())
          .and(environment_variables::app_id.is_null())
          .and(environment_variables::app_template_id.is_null()),
      )
      .or_filter(environment_variables::app_id.eq(app.id.clone()))
      .or_filter(
        environment_variables::app_template_id
          .eq(app.template.id.clone())
          .and(environment_variables::app_id.is_null())
          .and(environment_variables::workspace_id.is_null()),
      )
      .into_boxed();

    let env_vars = query
      .filter(environment_variables::archived_at.is_null())
      .load::<resource::EnvVar>(connection)
      .map_err(|e| {
        tracing::error!("{:?}", e);
      })
      .unwrap_or_default();

    let app_databases = databases::table
      .filter(
        databases::workspace_id
          .eq(workspace_id.to_string())
          .and(databases::app_id.eq(app.id.clone())),
      )
      .load::<Database>(connection)
      .map_err(|e| {
        tracing::error!("{:?}", e);
      })
      .unwrap_or_default();

    let app_database = app_databases.get(0);
    let database_cluster = match app_database {
      Some(db) if db.cluster_id.is_some() => {
        let clusters = database_clusters::table
          .filter(
            database_clusters::id
              .eq(db.cluster_id.as_ref().unwrap().to_string()),
          )
          .load::<DatabaseCluster>(connection)
          .map_err(|e| {
            tracing::error!("{:?}", e);
          })
          .unwrap_or_default();
        clusters.get(0).cloned()
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
            value: v.value.clone(),
            is_secret: false,
          },
        )
      })
      .collect::<HashMap<String, EnvVar>>();

    if let Some(db) = app_database {
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

    Ok(EnvironmentVariableStore::new(resources))
  }
}
