use anyhow::Result;
use serde_json::{json, Value};

pub(crate) fn from_vec<'a>(variables: Vec<Value>) -> Result<String> {
  Ok(format!(
    r#"
    class EnvironmentSecret {{
      constructor(id) {{
        this.id = id;
      }}
    }}

    const variables = Object.fromEntries({}.map(v => {{
      return [v.key, v.type == "secret" ? new EnvironmentSecret(v.id) : v.value]
    }}));

    export default variables;
    "#,
    json!(variables)
  ))
}
