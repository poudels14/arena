mod npm;

pub mod fs;

#[derive(Debug)]
pub struct ParsedSpecifier {
  package_name: String,
  sub_path: String,
}
