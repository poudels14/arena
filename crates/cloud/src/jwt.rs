use anyhow::{Context, Result};
use deno_core::{op2, OpState};
use jsonwebtoken::{
  decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize, Deserialize)]
struct JwtSignHeader {
  alg: Algorithm,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Claims {
  data: Value,
  /// Expiration time (as UTC timestamp)
  exp: usize,
  /// Issued at (as UTC timestamp)
  iat: Option<usize>,
  /// Not Before (as UTC timestamp)
  nbf: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtSignOptions {
  header: Header,
  payload: Value,
  secret: String,
}

#[op2]
#[string]
pub fn op_cloud_jwt_sign(
  _state: &mut OpState,
  #[serde] options: JwtSignOptions,
) -> Result<String> {
  encode(
    &options.header,
    &options.payload,
    &EncodingKey::from_secret((&options.secret).as_ref()),
  )
  .context("JWT encoding error")
}

#[op2]
#[serde]
pub fn op_cloud_jwt_verify(
  _state: &mut OpState,
  #[string] token: String,
  #[serde] algorith: Algorithm,
  #[string] secret: String,
) -> Result<serde_json::Value> {
  decode::<Value>(
    &token,
    &DecodingKey::from_secret((&secret).as_ref()),
    &Validation::new(algorith),
  )
  .context("JWT verification error")
  .map(|r| {
    json!({
        "header": r.header,
        "payload": r.claims,
    })
  })
}
