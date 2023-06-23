use super::response::HttpResponse;
use axum::response;
use http::{Response, StatusCode};
use hyper::body::HttpBody;
use hyper::Body;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Error {
  StdError,
  HyperError,
  HttpError,
  AnyhowError,
  ResponseBuilder,
  NotFound,
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {:?}", self)
  }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
  fn from(_: std::io::Error) -> Self {
    Self::StdError
  }
}

impl From<hyper::Error> for Error {
  fn from(_: hyper::Error) -> Self {
    Self::HyperError
  }
}

impl From<http::Error> for Error {
  fn from(_: http::Error) -> Self {
    Self::HttpError
  }
}

impl From<anyhow::Error> for Error {
  fn from(_: anyhow::Error) -> Self {
    Self::AnyhowError
  }
}

impl response::IntoResponse for Error {
  fn into_response(self) -> response::Response {
    match self {
      Self::NotFound => (StatusCode::NOT_FOUND, "Not found").into_response(),
      _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        .into_response(),
    }
  }
}

macro_rules! into_response {
  ($status: expr, $body: literal) => {
    Response::builder()
      .status($status)
      .body(Body::from($body).map_err(|_| unreachable!()).boxed_unsync())
      .unwrap()
  };
}

#[allow(dead_code)]
pub fn internal_server_error() -> HttpResponse {
  into_response!(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
}

#[allow(dead_code)]
pub fn not_found() -> HttpResponse {
  into_response!(StatusCode::NOT_FOUND, "Not found")
}
