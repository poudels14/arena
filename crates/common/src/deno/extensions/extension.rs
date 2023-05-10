use deno_core::Extension;
use std::path::PathBuf;

#[derive(Default)]
pub struct BuiltinExtension {
  pub extension: Option<Extension>,
  /// tuples of module's (specifier, path_to_source_file)
  /// these modules are loaded during snapshoting
  pub snapshot_modules: Vec<(&'static str, PathBuf)>,

  /// tuples of module's (specifier, source_code)
  /// these modules are loaded during runtime
  pub runtime_modules: Vec<(&'static str, &'static str)>,
}
