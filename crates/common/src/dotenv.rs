use std::path::Path;

pub use dotenvy::Result;

pub fn from_filename(filename: &Path) -> Result<Vec<(String, String)>> {
  dotenvy::from_filename_iter(filename).map(|envs| {
    envs
      .into_iter()
      .filter(|e| e.is_ok())
      .map(|e| e.unwrap())
      .collect()
  })
}
pub fn load_env(mode: &str, root: &Path) -> Result<Vec<(String, String)>> {
  if mode == "production" {
    dotenvy::from_filename_iter(root.join(".env"))
  } else {
    tracing::debug!("Loading .env.dev");
    dotenvy::from_filename_iter(root.join(".env.dev"))
  }
  .map(|envs| {
    envs
      .into_iter()
      .filter(|e| e.is_ok())
      .map(|e| e.unwrap())
      .collect()
  })
}
