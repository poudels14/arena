use super::request::HttpRequest;
use super::response::ParsedHttpResponse;
use deno_core::Resource;
use derivative::Derivative;
use futures::future::{RemoteHandle, Shared};
use std::borrow::Cow;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Default, Derivative)]
#[derivative(Debug)]
pub enum HttpServerConfig {
  Tcp {
    address: String,
    port: u16,
    serve_dir: Option<PathBuf>,
  },
  Stream(
    #[derivative(Debug = "ignore")]
    Rc<
      RefCell<
        mpsc::Receiver<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
      >,
    >,
  ),
  #[default]
  None,
}

impl Resource for HttpServerConfig {
  fn name(&self) -> Cow<str> {
    "httpServerConfig".into()
  }

  fn close(self: Rc<Self>) {}
}

#[derive(Clone)]
pub(super) struct TcpServer {
  pub listener: Rc<RefCell<TcpListener>>,
  pub serve_dir: Option<PathBuf>,
}

impl Resource for TcpServer {
  fn name(&self) -> Cow<str> {
    "tcpServer".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): close the service
  }
}

#[derive(Clone)]
pub(super) struct StreamServer {
  pub listener: Rc<
    RefCell<mpsc::Receiver<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>>,
  >,
}

impl Resource for StreamServer {
  fn name(&self) -> Cow<str> {
    "streamServer".into()
  }

  fn close(self: Rc<Self>) {}
}

#[derive(Clone)]
pub(super) struct HttpConnection {
  pub req_stream: Rc<
    RefCell<mpsc::Receiver<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>>,
  >,

  #[allow(dead_code)]
  pub closed_fut: Option<Shared<RemoteHandle<Result<(), Arc<hyper::Error>>>>>,
}

impl Resource for HttpConnection {
  fn name(&self) -> Cow<str> {
    "httpConnection".into()
  }

  fn close(self: Rc<Self>) {
    // TODO(sagar): close the service
  }
}

pub(super) struct HttpResponseHandle(
  pub RefCell<Option<oneshot::Sender<ParsedHttpResponse>>>,
);

impl Resource for HttpResponseHandle {
  fn name(&self) -> Cow<str> {
    "httpResponseHandle".into()
  }

  fn close(self: Rc<Self>) {}
}
