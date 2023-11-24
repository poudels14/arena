use derivative::Derivative;

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct RuntimeEnv {}

impl RuntimeEnv {
  pub fn new() -> Self {
    Self {}
  }
}
