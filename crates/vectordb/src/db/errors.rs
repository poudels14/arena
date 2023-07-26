use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
  Anyhow(anyhow::Error),
  NotFound(&'static str),
}

impl std::error::Error for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

impl From<anyhow::Error> for Error {
  fn from(e: anyhow::Error) -> Self {
    Self::Anyhow(e.into())
  }
}
