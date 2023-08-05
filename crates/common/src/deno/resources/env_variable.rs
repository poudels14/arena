use anyhow::{anyhow, Result};
use deno_core::{OpState, Resource};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EnvironmentVariable {
  /// The id of the secret is created randomly and only used to lookup secret
  /// env variables when the id is passed to rust by JS code in v8
  Secret {
    id: String,
    __type__: String,
  },
  Raw(Value),
}

impl Resource for EnvironmentVariable {
  fn close(self: Rc<Self>) {}
}

impl EnvironmentVariable {
  pub fn get_value(&self, state: &OpState) -> Result<Value> {
    match self {
      EnvironmentVariable::Raw(value) => Ok(value.to_owned()),
      EnvironmentVariable::Secret { id, __type__ } => {
        let variables = state.borrow::<EnvironmentVariableStore>().clone();
        variables
          .get(&id)
          .map(|e| e.value)
          .ok_or(anyhow!("Invalid secret environment variable"))
      }
    }
  }
}

#[derive(Clone, Debug)]
pub struct EnvVar {
  /// same as db row id
  pub id: String,
  pub app_template_id: Option<String>,
  pub app_id: Option<String>,
  pub key: String,
  pub value: Value,
  pub is_secret: bool,
}

#[derive(Clone, Debug)]
pub struct EnvironmentVariableStore(
  /// Map of temporary secret id to environemnt variable
  Rc<HashMap<String, EnvVar>>,
);

impl Resource for EnvironmentVariableStore {
  fn close(self: Rc<Self>) {}
}

impl EnvironmentVariableStore {
  pub fn new(vars: HashMap<String, EnvVar>) -> Self {
    Self(vars.into())
  }

  /// Return env variables matching `app_template_id` and `app_id`
  /// if they are set. Else, return env variables with
  /// env.app_id/app_template_id null
  pub fn filter(
    &self,
    app_id: Option<String>,
    app_template_id: Option<String>,
  ) -> Vec<Value> {
    self
      .0
      .iter()
      .filter(|ev| {
        app_template_id == ev.1.app_template_id && app_id == ev.1.app_id
      })
      .map(|(tmp_id, env)| {
        let app = match env.app_id.is_some() || env.app_template_id.is_some() {
          true => Some(json!({
              "id": env.app_id,
              "templateId": env.app_template_id,})),
          false => None,
        };

        json!({
          "id": env.id,
          "secretId": tmp_id,
          "app": app,
          "key": env.key,
          "isSecret": env.is_secret,
          "value": if env.is_secret { None } else { Some(env.value.clone()) }
        })
      })
      .collect::<Vec<Value>>()
  }

  pub fn get(&self, id: &str) -> Option<EnvVar> {
    self.0.get(id).and_then(|v| v.clone().into())
  }
}
