use crate::db::env_variable::{self, env_variables};
use anyhow::Result;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
pub struct RuntimeState {
  #[allow(dead_code)]
  workspace_id: String,
  pub env_variables: HashMap<String, env_variable::EnvVariable>,
}

/// This is the value that gets sent to v8
#[derive(Debug, Clone)]
pub struct EnvVariable {
  /// This id is created randomly and only used to lookup secret
  /// env variables when the id is passed to rust by JS code in v8
  pub tmp_id: String,
  // pub internal: ,
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
  ) -> Result<HashMap<String, env_variable::EnvVariable>> {
    let connection = &mut pool.get()?;
    Ok(
      env_variable::table
        .filter(env_variables::workspace_id.eq(workspace_id.to_string()))
        .load::<env_variable::EnvVariable>(connection)?
        .iter()
        .map(|v| (Uuid::new_v4().to_string(), v.clone()))
        .collect::<HashMap<String, env_variable::EnvVariable>>(),
    )
  }
}
