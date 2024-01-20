use std::collections::HashMap;

use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use runtime::env::{EnvVar, EnvironmentVariableStore};
use uuid::Uuid;

use super::app::App;
use super::MainModule;
use crate::db::resource::{self, resources};
use crate::loaders::registry::Registry;

#[derive(Debug, Clone)]
pub struct ArenaRuntimeState {
  pub workspace_id: String,
  pub registry: Registry,
  pub module: MainModule,
  pub env_variables: EnvironmentVariableStore,
}

impl ArenaRuntimeState {
  pub fn load_app_env_variables(
    workspace_id: &str,
    app: &App,
    connection: &mut PooledConnection<ConnectionManager<PgConnection>>,
  ) -> Result<EnvironmentVariableStore> {
    let query = resource::table
      .filter(
        resources::workspace_id
          .eq(workspace_id.to_string())
          .and(resources::app_id.is_null())
          .and(resources::app_template_id.is_null()),
      )
      .or_filter(resources::app_id.eq(app.id.clone()))
      .or_filter(
        resources::app_template_id
          .eq(app.template.id.clone())
          .and(resources::app_id.is_null())
          .and(resources::workspace_id.is_null()),
      )
      .into_boxed();

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
