use common::deno::extensions::server::response::HttpResponse;
use common::deno::extensions::server::HttpRequest;
use deno_core::Resource;
use std::borrow::Cow;
use std::rc::Rc;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct RequestStreamSender {
  pub(crate) sender: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
}

impl Resource for RequestStreamSender {
  fn name(&self) -> Cow<str> {
    "requestStreamSender".into()
  }

  fn close(self: Rc<Self>) {}
}
