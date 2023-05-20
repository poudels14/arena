use std::time::Instant;

use super::errors::Error;
use bytes::Bytes;
use http::Response;
use http_body::combinators::UnsyncBoxBody;

pub type HttpResponse = Response<UnsyncBoxBody<Bytes, Error>>;

pub struct HttpResponseMetata {
  pub method: String,
  pub path: String,
  pub req_received_at: Instant,
}
