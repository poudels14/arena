use super::template::Template;

#[derive(Debug, Clone)]
pub struct App {
  pub workspace_id: String,
  pub id: String,
  pub template: Template,
}
