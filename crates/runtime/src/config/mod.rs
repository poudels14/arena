mod arena;
pub mod node;

use std::net::IpAddr;
use std::path::PathBuf;

pub use arena::ArenaConfig;
use getset::Setters;
use tempfile::TempDir;

#[derive(Debug, Clone, Setters)]
#[getset(set = "pub")]
pub struct RuntimeConfig {
  /// Project root must be passed
  /// This should either be a directory where package.json is located
  /// or current directory
  /// Use {@link has_file_in_file_tree(Some(&cwd), "package.json")}
  /// to find the directory with package.json in file hierarchy
  /// Set this to random temp dir if the runtime doens't have
  /// access to file system
  pub project_root: PathBuf,

  /// The local address to use for outgoing network request
  /// This is useful if we need to restrict the outgoing network
  /// request to a specific network device/address
  pub egress_addr: Option<IpAddr>,

  pub process_args: Vec<String>,
}

impl Default for RuntimeConfig {
  fn default() -> Self {
    let temp_dir = TempDir::with_prefix("arena-runtime-")
      .expect("Failed to create temp dir for runtime");
    Self {
      project_root: temp_dir.into_path(),
      egress_addr: None,
      process_args: vec![],
    }
  }
}
