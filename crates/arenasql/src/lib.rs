mod ast;
mod error;

pub mod df;
pub mod runtime;
pub mod schema;
pub mod storage;

pub type Result<T> = std::result::Result<T, error::Error>;
pub type Error = error::Error;
