use anyhow::{anyhow, bail, Result};
use deno_core::error::AnyError;
use deno_core::{normalize_path, OpState};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use url::Url;

/// resolves the given path from project prefix/root and checks for
/// read permission. Returns Ok(resolved_path) if the permission
/// for given path is granted, else returns error
#[inline]
#[allow(dead_code)]
pub fn resolve_read_path(state: &mut OpState, path: &Path) -> Result<PathBuf> {
  let permissions = state.borrow_mut::<PermissionsContainer>();

  match permissions.fs.as_ref() {
    Some(perm) => {
      let resolved_path = resolve(&perm.root, path)?;
      permissions.check_read(&resolved_path)?;
      Ok(resolved_path)
    }
    None => bail!("No access to filesystem"),
  }
}

/// resolves the given path from project prefix/root and checks for
/// write permission. Returns Ok(resolved_path) if the permission
/// for given path is granted, else returns error
#[inline]
#[allow(dead_code)]
pub fn resolve_write_path(state: &mut OpState, path: &Path) -> Result<PathBuf> {
  let permissions = state.borrow_mut::<PermissionsContainer>();

  match permissions.fs.as_ref() {
    Some(perm) => {
      let resolved_path = resolve(&perm.root, path)?;
      permissions.check_write(&resolved_path)?;
      Ok(resolved_path)
    }
    None => bail!("No access to filesystem"),
  }
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TimerPermissions {
  pub allow_hrtime: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetPermissions {
  pub allowed_urls: Option<HashSet<Url>>,
  pub restricted_urls: Option<HashSet<Url>>,
}

#[allow(unused)]
impl NetPermissions {
  // Deny access to all urls
  pub fn restrict_all() -> Self {
    Self {
      allowed_urls: Some(Default::default()),
      ..Default::default()
    }
  }

  // Allow access to all urls
  pub fn allow_all() -> Self {
    Self {
      restricted_urls: Some(Default::default()),
      ..Default::default()
    }
  }

  // Only allow access to the list of urls
  pub fn only_allow(urls: HashSet<Url>) -> Self {
    Self {
      allowed_urls: Some(urls),
      ..Default::default()
    }
  }
}

#[derive(Derivative, Deserialize, Serialize)]
#[derivative(Clone, Debug, Default)]
pub struct FileSystemPermissions {
  /// The prefix that's used for the relative paths
  /// that are allowed for read/writes
  #[derivative(Default(value = "std::env::current_dir().unwrap()"))]
  pub root: PathBuf,
  // Note(sp): read paths are relative to the root
  pub allowed_read_paths: HashSet<String>,
  // Note(sp): read paths are relative to the root
  pub allowed_write_paths: HashSet<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct PermissionsContainer {
  pub timer: Option<TimerPermissions>,
  pub net: Option<NetPermissions>,

  /// File system permissions
  pub fs: Option<FileSystemPermissions>,
}

impl deno_fetch::FetchPermissions for PermissionsContainer {
  fn check_net_url(
    &mut self,
    url: &Url,
    _api_name: &str,
  ) -> Result<(), AnyError> {
    match &self.net.as_ref().and_then(|n| n.allowed_urls.as_ref()) {
      Some(allowed_urls) => {
        if allowed_urls.iter().any(|allowed| url_matches(allowed, url)) {
          Ok(())
        } else {
          Err(anyhow!("Domain not in allowed list"))
        }
      }
      None => match &self.net.as_ref().and_then(|n| n.restricted_urls.as_ref())
      {
        Some(restricted_urls) => {
          if restricted_urls
            .iter()
            .any(|restricted| url_matches(restricted, url))
          {
            Err(anyhow!("Restricted domain"))
          } else {
            Ok(())
          }
        }
        // Note: if neither allowed or restricted list is provided,
        // block all access
        None => Err(anyhow!("Network access restricted!")),
      },
    }
  }

  fn check_read(
    &mut self,
    _path: &Path,
    _api_name: &str,
  ) -> Result<(), AnyError> {
    // TODO(sagar)
    Ok(())
  }
}

fn url_matches(a: &Url, b: &Url) -> bool {
  // TODO(sagar): check port
  a.host_str() == b.host_str()
}

impl deno_web::TimersPermission for PermissionsContainer {
  fn allow_hrtime(&mut self) -> bool {
    self.timer.as_ref().and_then(|t| Some(t.allow_hrtime)) == Some(true)
  }
}

impl PermissionsContainer {
  /// Checks read access to a file path
  /// The path whose access is being checked should be absolute path
  ///
  /// Note(sagar): instead of using this directly,
  /// use `crate::deno::utils::fs::resolve_read_path`.
  pub fn check_read(&mut self, path: &Path) -> Result<()> {
    match self.fs.as_ref() {
      Some(perms) => {
        // Note(sagar): having permission to write to a path will also allow
        // reading from the path
        let allowed_read_paths = perms
          .allowed_write_paths
          .iter()
          .chain(perms.allowed_read_paths.iter())
          .collect::<Vec<&String>>();
        // TODO(sagar): cache the paths
        // TODO(sagar): write tests
        let root = perms.root.canonicalize()?;
        let path = normalize_path(path);

        if allowed_read_paths
          .iter()
          .any(|p| path.starts_with(normalize_path(root.join(p))))
        {
          return Ok(());
        }
      }
      None => {}
    };
    bail!(
      "doesn't have permission to read file: {}",
      path.to_string_lossy()
    )
  }

  /// Checks write access to a file path
  /// The path whose access is being checked should be absolute path
  ///
  /// Note(sagar): instead of using this directly,
  /// use `crate::deno::utils::fs::resolve_write_path`.
  #[allow(dead_code)]
  pub fn check_write(&mut self, path: &Path) -> Result<()> {
    match self.fs.as_ref() {
      Some(perms) => {
        // TODO(sagar): cache the paths
        // TODO(sagar): write tests
        let root = perms.root.canonicalize()?;
        let path = normalize_path(path);
        if perms
          .allowed_write_paths
          .iter()
          .any(|p| path.starts_with(normalize_path(root.join(p))))
        {
          return Ok(());
        }
      }
      None => {}
    }
    bail!(
      "doesn't have permission to write to file: {}",
      path.to_string_lossy()
    )
  }
}

/*********************************************************************/
/********************************* tests *****************************/
/*********************************************************************/
#[cfg(test)]
mod tests {
  use super::super::permissions::PermissionsContainer;
  use super::super::permissions::TimerPermissions;
  use deno_fetch::FetchPermissions;
  use deno_web::TimersPermission;
  use url::Url;

  #[test]
  fn test_timer_permissions_with_empty_permissions() {
    let mut permission = PermissionsContainer {
      ..Default::default()
    };
    assert_eq!(permission.allow_hrtime(), false);
  }

  #[test]
  fn test_timer_permissions_with_allow_hrtime_true() {
    let mut permission = PermissionsContainer {
      timer: Some(TimerPermissions { allow_hrtime: true }),
      ..Default::default()
    };
    assert_eq!(permission.allow_hrtime(), true);
  }

  #[test]
  fn test_timer_permissions_with_allow_hrtime_false() {
    let mut permission = PermissionsContainer {
      timer: Some(TimerPermissions {
        allow_hrtime: false,
      }),
      ..Default::default()
    };
    assert_eq!(permission.allow_hrtime(), false);
  }

  #[test]
  fn test_net_permissions_with_empty_permissions() {
    let mut permission = PermissionsContainer {
      timer: None,
      ..Default::default()
    };
    assert!(permission
      .check_net_url(&Url::parse("https://reqbin.com/echo").unwrap(), "")
      .is_err(),);
  }
}

#[inline]
fn resolve(base: &Path, path: &Path) -> Result<PathBuf> {
  if path.is_absolute() {
    Ok(normalize_path(path))
  } else {
    Ok(normalize_path(base.join(path)))
  }
}
