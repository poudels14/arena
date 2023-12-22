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
        let store = state.borrow::<EnvironmentVariableStore>().clone();
        store
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
  pub key: String,
  pub value: Value,
  pub is_secret: bool,
}

#[derive(Default, Clone, Debug)]
pub struct EnvironmentVariableStore(
  /// Map of temporary secret id to environemnt variable
  Rc<HashMap<String, EnvVar>>,
);

impl Resource for EnvironmentVariableStore {
  fn close(self: Rc<Self>) {}
}

#[allow(unused)]
impl EnvironmentVariableStore {
  pub fn new(vars: HashMap<String, EnvVar>) -> Self {
    Self(vars.into())
  }

  pub fn to_vec(&self) -> Vec<Value> {
    self
      .0
      .iter()
      .map(|(tmp_id, env)| {
        json!({
          "id": env.id,
          "secretId": tmp_id,
          "key": env.key,
          "isSecret": env.is_secret,
          "value": if env.is_secret {
            Some(Value::String("**secret**".to_string()))
          } else {
            Some(env.value.clone())
          }
        })
      })
      .collect::<Vec<Value>>()
  }

  pub fn get(&self, id: &str) -> Option<EnvVar> {
    self.0.get(id).and_then(|v| v.clone().into())
  }
}
