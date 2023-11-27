use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct RuntimeEnv {}

impl Default for RuntimeEnv {
  fn default() -> Self {
    Self {}
  }
}
