use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error {
  InvalidDimensionColumn,
}

impl std::error::Error for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match self {
      Self::InvalidDimensionColumn => {
        write!(f, "dimension column should be of type `vector(...)")
      }
    }
  }
}
