use super::template::Template;

#[derive(Debug, Clone)]
pub struct App {
  pub id: String,
  pub template: Template,
}
