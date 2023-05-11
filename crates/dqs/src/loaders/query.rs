use super::appkit::WidgetQuerySource;
use super::db::widget::widgets;
use super::AppkitModuleLoader;
use crate::loaders::db::widget;
use anyhow::Result;
use diesel::prelude::*;
use diesel::RunQueryDsl;

impl AppkitModuleLoader {
  pub async fn load_widget_query(
    &mut self,
    _workspace_id: String,
    source: &WidgetQuerySource,
  ) -> Result<String> {
    let connection = &mut self.pool.get()?;
    let widget = widgets::table
      .filter(widgets::id.eq(source.widget_id.to_string()))
      .first::<widget::Widget>(connection)?;

    println!("Widget loaded {:?}", widget);

    Ok(format!(
      r#"
        export default () => {{
          console.log('this is a query');
          return [1, 2, 3];
        }}
      "#
    ))
  }
}
