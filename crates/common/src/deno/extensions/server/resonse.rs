use super::errors::Error;
use bytes::Bytes;
use http::Response;
use http_body::combinators::UnsyncBoxBody;

pub type HttpResponse = Response<UnsyncBoxBody<Bytes, Error>>;
