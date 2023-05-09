use super::request::HttpRequest;
use deno_core::Resource;
use futures::future::{RemoteHandle, Shared};
use http::Response;
use hyper::Body;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

#[derive(Clone)]
pub(super) struct HttpServerConfig {
  pub address: String,
  pub port: u16,
}

impl Resource for HttpServerConfig {
  fn name(&self) -> Cow<str> {
    "httpServerConfig".into()
  }

  fn close(self: Rc<Self>) {}
}

#[derive(Clone)]
pub(super) struct HttpServer {
  pub address: String,
  pub port: u16,
  pub listener: Rc<RefCell<TcpListener>>,
}

impl Resource for HttpServer {
  fn name(&self) -> Cow<str> {
    "httpServer".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): close the service
  }
}

#[derive(Clone)]
pub(super) struct HttpConnection {
  pub req_stream:
    Rc<RefCell<mpsc::Receiver<(HttpRequest, mpsc::Sender<Response<Body>>)>>>,

  #[allow(dead_code)]
  pub closed_fut: Shared<RemoteHandle<Result<(), Arc<hyper::Error>>>>,
}

impl Resource for HttpConnection {
  fn name(&self) -> Cow<str> {
    "httpConnection".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): close the service
  }
}

pub(super) struct HttpResponseHandle {
  pub sender: mpsc::Sender<Response<Body>>,
}

impl Resource for HttpResponseHandle {
  fn name(&self) -> Cow<str> {
    "httpResponseHandle".into()
  }

  fn close(self: Rc<Self>) {}
}
