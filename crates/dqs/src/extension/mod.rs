use self::handle::DqsServerHandle;
use self::stream::RequestStreamSender;
use crate::server::{self, RuntimeConfig, ServerEvents};
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
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
use deno_core::StringOrBuffer;
use deno_core::ZeroCopyBuf;
use hyper::body::HttpBody;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
mod cluster;
mod handle;
mod stream;
use cluster::DqsCluster;

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
  Extension::builder("arena/runtime/dqs")
    .ops(vec![
      op_dqs_start_tcp_server::decl(),
      op_dqs_start_stream_server::decl(),
      op_dqs_list_servers::decl(),
      op_dqs_is_alive::decl(),
      op_dqs_ping::decl(),
      op_dqs_terminate_server::decl(),
      op_dqs_pipe_request_to_stream::decl(),
    ])
    .state(|state| {
      let cluster = DqsCluster::new();
      state.put::<DqsCluster>(cluster);
    })
    .build()
}

#[op]
async fn op_dqs_start_tcp_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
  address: String,
  port: u16,
) -> Result<ResourceId> {
  let mut cluster = state.borrow().borrow::<DqsCluster>().clone();
  let db_pool = cluster.get_db_pool()?;

  let (tx, rx) = oneshot::channel();
  let thread_handle = thread::spawn(move || {
    server::start(
      RuntimeConfig {
        workspace_id,
        db_pool: db_pool.into(),
        server_config: HttpServerConfig::Tcp(address.to_string(), port),
        ..Default::default()
      },
      tx,
    )
  });

  start_dqs_server(state, thread_handle, rx).await
}

#[op]
async fn op_dqs_start_stream_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
) -> Result<(ResourceId, ResourceId)> {
  let mut cluster = state.borrow().borrow::<DqsCluster>().clone();

  let (tx, rx) = oneshot::channel();
  let (stream_tx, stream_rx) = mpsc::channel(5);
  let db_pool = cluster.get_db_pool()?;
  let thread_handle = thread::spawn(move || {
    server::start(
      RuntimeConfig {
        workspace_id,
        db_pool: db_pool.into(),
        server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
          stream_rx,
        ))),
        ..Default::default()
      },
      tx,
    )
  });

  let handle_id = start_dqs_server(state.clone(), thread_handle, rx)
    .await
    .map_err(|_| anyhow!("Failed to spin up query runtime"))?;

  let sender_id = state
    .borrow_mut()
    .resource_table
    .add(RequestStreamSender { sender: stream_tx });

  Ok((handle_id, sender_id))
}

#[op]
async fn op_dqs_list_servers(
  state: Rc<RefCell<OpState>>,
) -> Result<Vec<ResourceId>> {
  let cluster = state.borrow().borrow::<DqsCluster>().clone();
  let cluster = cluster.borrow();
  Ok(cluster.instances.iter().map(|v| v.clone()).collect())
}

#[op]
fn op_dqs_is_alive(state: &mut OpState, handle_id: ResourceId) -> Result<bool> {
  Ok(state.resource_table.has(handle_id))
}

#[op]
async fn op_dqs_ping(
  state: Rc<RefCell<OpState>>,
  handle_id: ResourceId,
) -> Result<Value> {
  let handle = state
    .borrow()
    .resource_table
    .get::<DqsServerHandle>(handle_id)?;
  handle.commands.send(server::Command::Ping).await
}

#[op]
async fn op_dqs_terminate_server(
  state: Rc<RefCell<OpState>>,
  handle_id: ResourceId,
) -> Result<()> {
  let mut state = state.borrow_mut();
  if state.resource_table.has(handle_id) {
    let handle = state.resource_table.take::<DqsServerHandle>(handle_id)?;
    drop(state);
    handle.shutdown().await
  } else {
    bail!("DQS server not found")
  }
}

#[op]
async fn op_dqs_pipe_request_to_stream(
  state: Rc<RefCell<OpState>>,
  sender_id: ResourceId,
  req: (
    // url
    String,
    // method
    String,
    // headers
    Vec<(String, String)>,
    // body
    Option<StringOrBuffer>,
  ),
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
  sender
    .sender
    .send((
      HttpRequest {
        url: req.0,
        method: req.1,
        headers: req.2,
        body: req.3.map(|b| ZeroCopyBuf::ToV8(Some((*b.to_vec()).into()))),
      },
      tx,
    ))
    .await?;

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

async fn start_dqs_server(
  state: Rc<RefCell<OpState>>,
  thread_handle: JoinHandle<Result<()>>,
  rx: oneshot::Receiver<mpsc::Receiver<ServerEvents>>,
) -> Result<ResourceId> {
  let (handle_sender, handle_receiver) = oneshot::channel();
  let mut handle_sender = Some(handle_sender);
  let mut thread_handle = Some(thread_handle);
  tokio::task::spawn_local(async move {
    let mut receiver = rx
      .await
      .context("Error listening to DQS server events")
      .unwrap();

    let mut handle_id = None;
    let x = loop {
      match receiver.recv().await {
        Some(ServerEvents::Started(isolate_handle, commands))
          if handle_id == None =>
        {
          let mut cluster = state.borrow().borrow::<DqsCluster>().clone();
          let hid = cluster
            .add_instance(
              &mut state.borrow_mut(),
              DqsServerHandle {
                isolate_handle,
                thread_handle: thread_handle.take().unwrap(),
                commands,
              },
            )
            .unwrap();
          handle_sender
            .take()
            .map(|tx| tx.send(Ok(hid)).unwrap())
            .unwrap();
          handle_id = Some(hid);
        }
        Some(ServerEvents::Terminated(result)) => {
          let mut cluster = state.borrow().borrow::<DqsCluster>().clone();
          break handle_id
            .map(|id| cluster.remove_instance(id).unwrap())
            .ok_or(anyhow!("Failed to clean up server instance"))
            .and_then(|_| result);
        }
        _ => {
          break Err(anyhow!("Server events stream closed"));
        }
      }
    };
    handle_sender
      .take()
      .and_then(|tx| tx.send(Err(x.unwrap_err())).ok())
      .unwrap();
  });

  handle_receiver.await?
}
