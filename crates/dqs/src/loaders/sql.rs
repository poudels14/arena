use crate::types::widget::{SqlSourceConfig, WidgetQuerySpecifier};
use anyhow::{anyhow, Result};
use handlebars::Handlebars;
use serde_json::json;

pub(crate) fn from_config<'a>(
  _specifier: &WidgetQuerySpecifier,
  config: &SqlSourceConfig,
) -> Result<String> {
  let mut reg = Handlebars::new();
  reg.set_strict_mode(true);
  reg
    .render_template(
      r#"
      import { connect } from "@arena/core/dqs/postgres";
      import env from "./env";

      export default async function() {
        return await connect({
          connectionString: env['{{db}}'],
        }).execute(
          `{{query}}`,
          // TODO(sagar): parameterize the query
          []);
      }
      "#,
      &json!({
        "db": config.db,
        "query": config.query
      }),
    )
    .map_err(|e| anyhow!("{:?}", e))
}
