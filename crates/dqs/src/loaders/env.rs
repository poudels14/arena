use anyhow::Result;
use serde_json::{json, Value};

pub(crate) fn to_esm_module<'a>(variables: Vec<Value>) -> Result<String> {
  Ok(format!(
    r#"
    class EnvironmentSecret {{
      constructor(id) {{
        this.id = id;
        this.__type__ = "secret";
        Object.freeze(this);
      }}
    }}

    const variables = Object.fromEntries({}.flatMap(v => {{
      const value = v.isSecret ? new EnvironmentSecret(v.secretId) : v.value;
      return [
        [v.key, value],
      ];
    }}));

    export default variables;
    "#,
    json!(variables)
  ))
}
