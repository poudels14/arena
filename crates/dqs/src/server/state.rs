use crate::apps::App;
use crate::db::resource::{self, resources};
use anyhow::Result;
use common::deno::resources::env_variable::{EnvVar, EnvironmentVariableStore};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuntimeState {
  pub env_variables: EnvironmentVariableStore,
}

impl RuntimeState {
  pub async fn init(
    workspace_id: String,
    app: &Option<App>,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
  ) -> Result<Self> {
    let env_variables =
      Self::load_env_variables(&workspace_id, app, connection)?;

    Ok(Self { env_variables })
  }

  fn load_env_variables(
    workspace_id: &str,
    app: &Option<App>,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
  ) -> Result<EnvironmentVariableStore> {
    let mut query = resource::table
      .filter(
        resources::workspace_id
          .eq(workspace_id.to_string())
          .and(resources::app_id.is_null())
          .and(resources::app_template_id.is_null()),
      )
      .into_boxed();

    query = match app {
      Some(app) => query
        .or_filter(resources::app_id.eq(app.id.clone()))
        .or_filter(
          resources::app_template_id
            .eq(app.template.id.clone())
            .and(resources::app_id.is_null())
            .and(resources::workspace_id.is_null()),
        ),
      None => query,
    };

    let resources = query
      .filter(resources::archived_at.is_null())
      .load::<resource::Resource>(connection)?
      .iter()
      .map(|v| {
        (
          Uuid::new_v4().to_string(),
          EnvVar {
            id: v.id.clone(),
            key: v.key.clone(),
            value: v.value.clone(),
            is_secret: v.secret,
          },
        )
      })
      .collect::<HashMap<String, EnvVar>>();
    Ok(EnvironmentVariableStore::new(resources))
  }
}
