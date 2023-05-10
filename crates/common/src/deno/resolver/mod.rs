mod config;
mod npm;

pub mod fs;
pub use config::Config;

#[derive(Debug)]
pub struct ParsedSpecifier {
  package_name: String,
  sub_path: String,
}
