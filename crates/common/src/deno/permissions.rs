use anyhow::{anyhow, bail, Result};
use deno_core::error::AnyError;
use deno_core::{normalize_path, OpState};
use derivative::Derivative;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct TimerPermissions {
  pub allow_hrtime: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FetchPermissions {
  pub allowed_urls: Option<HashSet<Url>>,
  pub restricted_urls: Option<HashSet<Url>>,
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
  pub net: Option<FetchPermissions>,

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
  fn check_unstable(&self, _: &OpState, _: &'static str) {}
}

impl PermissionsContainer {
  /// Checks read access to a file path
  pub fn check_read(&mut self, path: &Path) -> Result<()> {
    match self.fs.as_ref() {
      Some(perms) => {
        // TODO(sagar): cache the paths
        // TODO(sagar): write tests
        let root = perms.root.canonicalize()?;
        let path = path.canonicalize()?;
        if perms
          .allowed_read_paths
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
  #[allow(dead_code)]
  pub fn check_write(&mut self, path: &Path) -> Result<()> {
    match self.fs.as_ref() {
      Some(perms) => {
        // TODO(sagar): cache the paths
        // TODO(sagar): write tests
        let root = perms.root.canonicalize()?;
        let path = path.canonicalize()?;
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
