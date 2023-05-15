use self::handle::DqsServerHandle;
use self::stream::RequestStreamSender;
use crate::runtime::RuntimeConfig;
use crate::server;
use crate::server::ServerEvents;
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
use hyper::body::HttpBody;
use serde_json::Value;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread;
use std::thread::JoinHandle;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
mod handle;
mod servers;
mod stream;
use servers::DqsServers;

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
      op_dqs_list_servers::decl(),
      op_dqs_ping::decl(),
      op_dqs_terminate_server::decl(),
      op_dqs_pipe_request_to_stream::decl(),
    ])
    .state(|state| {
      let servers = DqsServers::new();
      state.put::<DqsServers>(servers);
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

  start_dqs_server(state, thread_handle, rx).await
}

#[op]
async fn op_dqs_start_stream_server(
  state: Rc<RefCell<OpState>>,
  workspace_id: String,
) -> Result<(ResourceId, ResourceId)> {
  let (tx, rx) = oneshot::channel();
  let (stream_tx, stream_rx) = mpsc::channel(5);
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

  let handle_id = start_dqs_server(state.clone(), thread_handle, rx).await?;
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
  let servers = state.borrow().borrow::<DqsServers>().clone();
  let servers = servers.borrow();
  Ok(servers.instances.iter().map(|v| v.clone()).collect())
}

#[op]
/// This can be used to check if the DQS server thread is running
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
    handle.shutdown().await
  } else {
    bail!("DQS server not found")
  }
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

async fn start_dqs_server(
  state: Rc<RefCell<OpState>>,
  thread_handle: JoinHandle<Result<()>>,
  rx: oneshot::Receiver<mpsc::Receiver<ServerEvents>>,
) -> Result<ResourceId> {
  let mut receiver = rx.await?;

  if let Some(event) = receiver.recv().await {
    match event {
      ServerEvents::Started(isolate_handle, commands) => {
        let handle_id = {
          let mut servers = state.borrow().borrow::<DqsServers>().clone();
          servers.add_instance(
            &mut state.borrow_mut(),
            DqsServerHandle {
              isolate_handle,
              thread_handle,
              commands,
            },
          )?
        };

        tokio::task::spawn_local(async move {
          let mut servers = state.borrow().borrow::<DqsServers>().clone();
          loop {
            match receiver.recv().await {
              Some(ServerEvents::Terminated) => {
                servers.remove_instance(handle_id).unwrap();
                return;
              }
              _ => {}
            }
          }
        });
        return Ok(handle_id);
      }
      _ => {}
    }
  }
  bail!("error starting new DQS server")
}
