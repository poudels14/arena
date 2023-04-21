use deno_core::Resource;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::rc::Rc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecretResource {
  pub value: Value,
}

impl Resource for SecretResource {
  fn close(self: Rc<Self>) {}
}
