use crate::db::env_variable::{self, env_variables};
use anyhow::Result;
use common::deno::resources::env_variable::{EnvVar, EnvironmentVariableStore};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuntimeState {
  #[allow(dead_code)]
  pub workspace_id: String,
  pub env_variables: EnvironmentVariableStore,
}

impl RuntimeState {
  pub async fn init(
    workspace_id: String,
    pool: Pool<ConnectionManager<PgConnection>>,
  ) -> Result<Self> {
    let env_variables = Self::load_env_variables(&workspace_id, &pool)?;

    Ok(Self {
      workspace_id,
      env_variables,
    })
  }

  fn load_env_variables(
    workspace_id: &str,
    pool: &Pool<ConnectionManager<PgConnection>>,
  ) -> Result<EnvironmentVariableStore> {
    let connection = &mut pool.get()?;
    Ok(EnvironmentVariableStore(
      env_variable::table
        .filter(env_variables::workspace_id.eq(workspace_id.to_string()))
        .load::<env_variable::EnvVariable>(connection)?
        .iter()
        .map(|v| {
          (
            Uuid::new_v4().to_string(),
            EnvVar {
              id: v.id.clone(),
              key: v.key.clone(),
              value: v.value.clone(),
              is_secret: v.ttype == "secret",
            },
          )
        })
        .collect::<HashMap<String, EnvVar>>()
        .into(),
    ))
  }
}
