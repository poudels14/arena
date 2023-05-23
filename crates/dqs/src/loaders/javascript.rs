use crate::types::widget::JavascriptSourceConfig;
use crate::types::widget::WidgetQuerySpecifier;
use anyhow::anyhow;
use anyhow::Result;
use handlebars::{no_escape, Handlebars};
use once_cell::sync::Lazy;
use serde_json::json;

static TEMPLATE: Lazy<Result<Handlebars>> = Lazy::new(|| {
  let mut reg = Handlebars::new();
  reg.set_strict_mode(true);
  reg.register_escape_fn(no_escape);
  reg.register_template_string(
    "JS_QUERY_MODULE",
    include_str!("./js-template.js"),
  )?;

  Ok(reg)
});

pub(crate) fn from_config<'a>(
  _specifier: &WidgetQuerySpecifier,
  config: &JavascriptSourceConfig,
) -> Result<String> {
  TEMPLATE
    .as_ref()
    .expect("failed to load query template")
    .render(
      "JS_QUERY_MODULE",
      &json!({
        "jsQuery": config.value
      }),
    )
    .map_err(|e| anyhow!("{:?}", e))
}
