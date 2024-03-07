use std::collections::HashMap;

use anyhow::Result;
use derivative::Derivative;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use serde_json::Value;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::app::App;
use super::MainModule;
use crate::db::database;
use crate::db::resource;
use crate::loaders::registry::Registry;

#[derive(Derivative)]
#[derivative(Debug, Clone)]
pub struct ArenaRuntimeState {
  pub workspace_id: String,
  pub registry: Registry,
  pub module: MainModule,
  pub env_variables: EnvironmentVariableStore,
}

impl ArenaRuntimeState {
  #[tracing::instrument(skip_all, err, level = "debug")]
  pub async fn load_app_env_variables(
    workspace_id: &str,
    app: &App,
    pool: &Pool<Postgres>,
  ) -> Result<EnvironmentVariableStore> {
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

    Ok(EnvironmentVariableStore::new(resources))
  }
}
