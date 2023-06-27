use std::path::Path;

pub fn load_env(mode: &str, root: &Path) -> Option<Vec<(String, String)>> {
  if mode == "production" {
    dotenvy::from_filename_iter(root.join(".env")).ok()
  } else {
    dotenvy::from_filename_iter(root.join(".env.dev")).ok()
  }
  .map(|envs| {
    envs
      .into_iter()
      .filter(|e| e.is_ok())
      .map(|e| e.unwrap())
      .collect()
  })
}
