use common::deno::extensions::server::response::ParsedHttpResponse;
use common::deno::extensions::server::HttpRequest;
use deno_core::Resource;
use std::borrow::Cow;
use std::rc::Rc;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub struct RequestStreamSender {
  pub(crate) sender:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

impl Resource for RequestStreamSender {
  fn name(&self) -> Cow<str> {
    "requestStreamSender".into()
  }

  fn close(self: Rc<Self>) {}
}
