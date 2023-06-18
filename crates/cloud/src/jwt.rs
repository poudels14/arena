use anyhow::{Context, Result};
use deno_core::{op, OpState};
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
struct JwtSignOptions {
  header: Header,
  payload: Value,
  secret: String,
}

#[op]
fn op_cloud_jwt_sign(
  _state: &mut OpState,
  options: JwtSignOptions,
) -> Result<String> {
  encode(
    &options.header,
    &options.payload,
    &EncodingKey::from_secret((&options.secret).as_ref()),
  )
  .context("JWT encoding error")
}

#[op]
fn op_cloud_jwt_verify(
  _state: &mut OpState,
  token: String,
  algorith: Algorithm,
  secret: String,
) -> Result<Value> {
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
