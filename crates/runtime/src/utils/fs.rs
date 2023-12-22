use std::path::Path;
use std::path::PathBuf;

/// Checks whether there's a file with the given name in the given directory
/// or any of it's ancestors up the file tree. For example, if dir = /a/b/c
/// and filename = f.js, this will check if any of these files exist:
///  - /a/b/c/f.js
///  - /a/b/f.js
///  - /a/f.js
///  - /f.js
///
/// If a file exist, it will return the directory in which the file is found.
pub fn has_file_in_file_tree(
  dir: Option<&Path>,
  filename: &str,
) -> Option<PathBuf> {
  let mut dir = dir;
  loop {
    if let Some(d) = dir {
      if d.join(filename).exists() {
        return Some(d.to_path_buf());
      }
      dir = d.parent();
    } else {
      return None;
    }
  }
}
