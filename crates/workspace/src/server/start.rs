use super::events::{ServerEvent, ServerEvents, ServerStarted};
use super::ext::ResponseSender;
use super::http::HttpRequest;
use crate::server::{
  ClientRequest, ServerHandle, ServerOptions, ServerRequest,
  WorkspaceServerHandle,
};
use crate::Workspace;
use anyhow::Result;
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use log::{debug, error, info};
use serde_json::{json, Value};
use std::sync::Arc;
use std::thread;
use tokio::sync::{mpsc, Mutex};
use tokio::task;

/// This is a handle that's used to send parsed TCP requests to JS VM
#[derive(Clone, Debug)]
pub(crate) struct VmService {
  pub sender: mpsc::Sender<(HttpRequest, ResponseSender)>,
}

#[derive(Clone, Debug)]
pub(crate) struct WorkspaceServer {
  /// Socket address of the workspace
  pub address: String,

  /// The port workspace server is listening to
  pub port: u16,

  /// Workspace config
  pub(crate) workspace: Workspace,

  /// Message handle that can be used to communicate with the server
  pub(crate) handle: ServerHandle<ServerRequest, ClientRequest>,

  /// Handle for events emitted by the server
  pub(crate) events: ServerEvents,

  /// A service that exchanges messages/requests with VM
  pub(crate) vm_serice: Option<VmService>,
}

/// Start a server to serve a workspace. The server is single threaded
/// because v8 only supports single thread.
///
/// This is non-blocking, so the caller need to call wait_for_termination
/// manually to wait until the server is terminated
pub async fn serve(
  workspace: Workspace,
  options: ServerOptions,
) -> Result<WorkspaceServerHandle> {
  let server = WorkspaceServer {
    workspace: workspace.clone(),
    address: options.address,
    port: options.port,
    handle: ServerHandle::new(),
    events: ServerEvents::new(),
    vm_serice: None,
  };

  let thread = thread::Builder::new()
    .name(format!("arena-workspace-server-{}", workspace.config.name));

  let mut server_clone = server.clone();
  thread.spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .enable_time()
      .worker_threads(2)
      // TODO(sagar): optimize max blocking threads
      .max_blocking_threads(2)
      .on_thread_start(|| println!("Tokio runtime started...."))
      .on_thread_stop(|| println!("Tokio runtime stopped..."))
      .build()
      .unwrap();

    rt.block_on(async {
      server_clone.start_all().await.unwrap();
    });
  })?;

  // Note(sagar): we need to update address/port since the server uses
  // a random port by default
  let metadata: ServerStarted = serde_json::from_value(
    server.events.wait_until(ServerEvent::Started).await?,
  )
  .unwrap();

  let handle = WorkspaceServerHandle {
    workspace: server.workspace,
    address: metadata.address,
    port: metadata.port,
    handle: server.handle,
    events: server.events,
  };

  Ok(handle)
}

impl WorkspaceServer {
  async fn start_all(&mut self) -> Result<()> {
    info!(
      "Workspace [name={}] server started...",
      self.workspace.config.name
    );

    let (tx, rx) = mpsc::channel::<(HttpRequest, ResponseSender)>(100);
    self.vm_serice = Some(VmService { sender: tx });
    let mut js_runtime = self.start_workspace_js_server(rx).await?;

    // Note(sagar): in a adhoc benchmarking, running tcp server in localset
    // performed better than without it. probably due to async locking on
    // message channels
    let local = task::LocalSet::new();
    let tcp_server =
      local.run_until(async { super::http::listen(self.clone()).await });

    let command_listener = self.listen_to_admin_commands();

    tokio::select! {
      c = command_listener => {
        debug!("admin command listener terminated: {:?}", c);
      },
      c = tcp_server => {
        match c {
          Err(e) => error!("TCP server terminated with error: {:?}", e),
          _ => debug!("TCP server terminated.")
        }
      }
      c = js_runtime.run_event_loop() => {
        match c {
          Err(e) => error!("Js runtime terminated with error: {:?}", e),
          _ => debug!("JS runtime terminated.")
        }
      }
    }

    info!(
      "Workspace [name={}] server terminated",
      self.workspace.config.name
    );
    self
      .events
      .sender
      .send((ServerEvent::Terminated, Value::Null))?;

    Ok(())
  }

  async fn listen_to_admin_commands(&self) -> Result<()> {
    let handle = self.handle.clone();
    let mut rx = handle.client.rx.lock().await;
    loop {
      while let Some((req, res)) = rx.recv().await {
        debug!("Admin command received: {:?}", req);
        res
          .send(json!({
            "data": {
              "success": true,
            }
          }))
          .unwrap();
      }
    }
  }

  async fn start_workspace_js_server(
    &self,
    rx: mpsc::Receiver<(HttpRequest, ResponseSender)>,
  ) -> Result<IsolatedRuntime> {
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      transpile: true,
      extensions: vec![super::ext::init(Arc::new(Mutex::new(rx)))],
      ..Default::default()
    });

    runtime.execute_script(
      "",
      &format!(
        r#"
        import("file://{}").then(async ({{ default: m }}) => {{
          Arena.Workspace.handleRequest(async (req) => {{
            let res = m.execute(req);
            if (res.then) {{
              res = await res;
            }}
            return res;
          }});
        }});
      "#,
        self.workspace.entry_file().to_str().unwrap(),
      ),
    )?;
    Ok(runtime)
  }
}
