use anyhow::Result;
use serde_json::{json, Value};

pub(crate) fn from_vec<'a>(variables: Vec<Value>) -> Result<String> {
  Ok(format!(
    r#"
    class EnvironmentSecret {{
      constructor(id) {{
        this.id = id;
        this.__type__ = "secret";
        Object.freeze(this);
      }}
    }}

    const variables = Object.fromEntries({}.map(v => {{
      return [v.key, v.isSecret ? new EnvironmentSecret(v.id) : v.value]
    }}));

    export default variables;
    "#,
    json!(variables)
  ))
}
