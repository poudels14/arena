use std::collections::BTreeMap;

use anyhow::{anyhow, Context, Result};
use axum_extra::extract::cookie::Cookie;
use cloud::identity::Identity;
use http::Request;
use hyper::Body;
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use runtime::extensions::server::errors;
use serde_json::Value;

use super::cache::Cache;
use crate::arena::App;

/// Returns tuple of (identity, App) if authorization succeeds
pub async fn authenticate_user_using_headers(
  cache: &Cache,
  jwt_secret: &str,
  app_id: &str,
  req: &Request<Body>,
) -> Result<(Identity, App), errors::Error> {
  let identity =
    parse_identity_from_header(jwt_secret, req).unwrap_or(Identity::Unknown);
  tracing::trace!("identity = {:?}", identity);

  authenticate_user(cache, app_id, &identity)
    .await
    .map(|app| (identity, app))
}

/// Returns App if authorization succeeds
#[tracing::instrument(skip_all, err, level = "trace")]
pub async fn authenticate_user(
  cache: &Cache,
  app_id: &str,
  #[allow(unused)] identity: &Identity,
) -> Result<App, errors::Error> {
  let app = cache
    .get_app(app_id)
    .await
    .map_err(|e| {
      tracing::error!("Error getting workspace id: {}", e);
      errors::Error::AnyhowError(e.to_string())
    })?
    .ok_or(errors::Error::NotFound)?;
  tracing::trace!("app = {:?}", app);

  #[cfg(not(feature = "disable-auth"))]
  {
    let acl_checker =
      cache.get_app_acl_checker(&app.id).await.unwrap_or_default();
    let has_access = match identity {
      Identity::User { ref id, .. } => acl_checker.read().has_any_access(&id),
      Identity::App { id, .. } => cache
        .get_app(id)
        .await?
        .and_then(|app| app.owner_id)
        .map(|owner_id| acl_checker.read().has_any_access(&owner_id))
        .unwrap_or(false),
      Identity::Unknown => acl_checker.read().has_any_access("public"),
      _ => false,
    };

    if !has_access {
      tracing::trace!("doesn't have access");
      return Err(errors::Error::Forbidden);
    }
  }
  Ok(app)
}

fn parse_cookies(req: &Request<Body>) -> BTreeMap<String, String> {
  Cookie::split_parse(
    req
      .headers()
      .get("cookie")
      .and_then(|c| c.to_str().ok())
      .unwrap_or_default(),
  )
  .into_iter()
  .fold(BTreeMap::new(), |mut map, c| {
    if let Ok(cookie) = c {
      map.insert(cookie.name().to_string(), cookie.value().to_string());
    }
    map
  })
}

pub fn parse_identity_from_header(
  jwt_secret: &str,
  req: &Request<Body>,
) -> Result<Identity> {
  let cookies = { parse_cookies(req) };
  let token = cookies.get("user").map(|v| v.as_str()).or_else(|| {
    req
      .headers()
      .get("x-portal-authentication")
      .and_then(|c| c.to_str().ok())
  });

  if token.is_none() {
    return Ok(Identity::Unknown);
  }
  let token = token.unwrap();

  jsonwebtoken::decode::<Value>(
    &token,
    &DecodingKey::from_secret(jwt_secret.as_bytes()),
    &Validation::new(Algorithm::HS512),
  )
  .context("JWT verification error")
  .and_then(|mut r| {
    let claims = r
      .claims
      .as_object_mut()
      .ok_or(anyhow!("Invalid JWT token"))?;

    // when deserializing enum, can't have unspecified fields
    claims.retain(|k, _| k == "user" || k == "app" || k == "workflowRun");

    serde_json::from_value(r.claims)
      .context("Failed to parse identity from header")
  })
}
