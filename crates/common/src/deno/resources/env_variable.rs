use anyhow::{anyhow, Result};
use deno_core::{OpState, Resource};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
  pub key: String,
  pub value: Value,
  pub is_secret: bool,
}

#[derive(Clone, Debug)]
pub struct EnvironmentVariableStore(
  /// Map of temporary secret id to environemnt variable
  pub Rc<HashMap<String, EnvVar>>,
);

impl Resource for EnvironmentVariableStore {
  fn close(self: Rc<Self>) {}
}

impl EnvironmentVariableStore {
  pub fn get(&self, id: &str) -> Option<EnvVar> {
    self.0.get(id).and_then(|v| v.clone().into())
  }
}
