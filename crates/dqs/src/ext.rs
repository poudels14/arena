use crate::runtime::RuntimeConfig;
use crate::server;
use crate::server::DqsServerHandle;
use anyhow::bail;
use anyhow::Result;
use common::deno::extensions::server::HttpRequest;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::BuiltinExtension;
use common::resolve_from_root;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::Resource;
use deno_core::ResourceId;
use deno_core::ZeroCopyBuf;
use http::Response;
use hyper::body::HttpBody;
use hyper::Body;
use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/dqs",
      resolve_from_root!("../../js/arena-runtime/dist/dqs.js"),
    )],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("@arena/dqs")
    .ops(vec![
      op_dqs_start_tcp_server::decl(),
      op_dqs_start_stream_server::decl(),
      op_dqs_pipe_request_to_stream::decl(),
    ])
    .build()
}

#[op]
async fn op_dqs_start_tcp_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
  address: String,
  port: u16,
) -> Result<ResourceId> {
  let (tx, rx) = oneshot::channel();
  let thread_handle = thread::spawn(move || {
    server::start(
      RuntimeConfig {
        workspace_id,
        server_config: HttpServerConfig::Tcp(address.to_string(), port),
        ..Default::default()
      },
      tx,
    )
  });

  let isolate_handle = rx.await?;
  let resource_id = state.borrow_mut().resource_table.add(DqsServerHandle {
    isolate_handle,
    thread_handle,
  });
  Ok(resource_id)
}

#[op]
async fn op_dqs_start_stream_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
) -> Result<(ResourceId, ResourceId)> {
  let (tx, rx) = oneshot::channel();
  let (stream_tx, stream_rx) = mpsc::channel(10);
  let thread_handle = thread::spawn(move || {
    server::start(
      RuntimeConfig {
        workspace_id,
        server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
          stream_rx,
        ))),
        ..Default::default()
      },
      tx,
    )
  });

  let isolate_handle = rx.await?;
  let handle_id = state.borrow_mut().resource_table.add(DqsServerHandle {
    isolate_handle,
    thread_handle,
  });

  let sender_id = state
    .borrow_mut()
    .resource_table
    .add(RequestStreamSender { sender: stream_tx });
  Ok((handle_id, sender_id))
}

#[op]
async fn op_dqs_pipe_request_to_stream(
  state: Rc<RefCell<OpState>>,
  sender_id: ResourceId,
  request: HttpRequest,
) -> Result<(
  u16,                   /* status */
  Vec<(String, String)>, /* headers */
  ZeroCopyBuf,           /* body */
)> {
  let sender = state
    .borrow()
    .resource_table
    .get::<RequestStreamSender>(sender_id)?;

  let (tx, mut rx) = mpsc::channel(2);
  sender.sender.send((request, tx)).await?;

  match rx.recv().await {
    Some(mut response) => Ok((
      response.status().into(),
      response
        .headers()
        .iter()
        .map(|(key, value)| {
          (
            key.to_string(),
            String::from_utf8(value.as_bytes().to_owned()).unwrap(),
          )
        })
        .collect::<Vec<(String, String)>>(),
      <Box<[u8]> as Into<ZeroCopyBuf>>::into(
        response
          .body_mut()
          .data()
          .await
          .and_then(|r| r.ok())
          .map(|r| r.to_vec().into_boxed_slice())
          .unwrap_or_default(),
      ),
    )),
    _ => bail!("error receiving response from stream"),
  }
}

#[derive(Clone)]
pub struct RequestStreamSender {
  sender: mpsc::Sender<(HttpRequest, mpsc::Sender<Response<Body>>)>,
}

impl Resource for RequestStreamSender {
  fn name(&self) -> Cow<str> {
    "requestStreamSender".into()
  }

  fn close(self: Rc<Self>) {}
}
