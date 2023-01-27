use anyhow::anyhow;
use deno_core::error::AnyError;
use deno_core::OpState;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::Path;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerPermissions {
  pub allow_hrtime: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchPermissions {
  pub allowed_urls: Option<HashSet<Url>>,
  pub restricted_urls: Option<HashSet<Url>>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
  pub timer: Option<TimerPermissions>,
  pub net: Option<FetchPermissions>,
}

impl deno_fetch::FetchPermissions for Permissions {
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

impl deno_web::TimersPermission for Permissions {
  fn allow_hrtime(&mut self) -> bool {
    self.timer.as_ref().and_then(|t| Some(t.allow_hrtime)) == Some(true)
  }
  fn check_unstable(&self, _: &OpState, _: &'static str) {}
}

/*********************************************************************/
/********************************* tests *****************************/
/*********************************************************************/
#[cfg(test)]
mod tests {
  use crate::permissions::Permissions;
  use crate::permissions::TimerPermissions;
  use deno_fetch::FetchPermissions;
  use deno_web::TimersPermission;
  use url::Url;

  #[test]
  fn test_timer_permissions_with_empty_permissions() {
    let mut permission = Permissions {
      timer: None,
      net: None,
    };
    assert_eq!(permission.allow_hrtime(), false);
  }

  #[test]
  fn test_timer_permissions_with_allow_hrtime_true() {
    let mut permission = Permissions {
      timer: Some(TimerPermissions { allow_hrtime: true }),
      net: None,
    };
    assert_eq!(permission.allow_hrtime(), true);
  }

  #[test]
  fn test_timer_permissions_with_allow_hrtime_false() {
    let mut permission = Permissions {
      timer: Some(TimerPermissions {
        allow_hrtime: false,
      }),
      net: None,
    };
    assert_eq!(permission.allow_hrtime(), false);
  }

  #[test]
  fn test_net_permissions_with_empty_permissions() {
    let mut permission = Permissions {
      timer: None,
      net: None,
    };
    assert!(permission
      .check_net_url(&Url::parse("https://reqbin.com/echo").unwrap(), "")
      .is_err(),);
  }
}
