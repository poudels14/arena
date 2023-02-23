use super::events::{ServerEvent, ServerEvents, ServerStarted};
use super::ext::ResponseSender;
use super::http::HttpRequest;
use crate::server::{
  ClientRequest, ServerHandle, ServerOptions, ServerRequest,
  WorkspaceServerHandle,
};
use crate::Workspace;
use anyhow::{anyhow, bail, Result};
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashSet;
#[cfg(debug_assertions)]
use std::path::Path;
use std::rc::Rc;
use std::thread;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{debug, error, info};
use url::Url;

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
    .name(format!("workspace-[{}]", workspace.config.name));

  let mut server_clone = server.clone();
  thread.spawn(move || {
    let rt = tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .enable_time()
      .worker_threads(1)
      // TODO(sagar): optimize max blocking threads
      .max_blocking_threads(2)
      .build()
      .unwrap();

    rt.block_on(async {
      match server_clone.start_all().await {
        Err(e) => {
          server_clone
            .events
            .sender
            .send((ServerEvent::Terminated, Value::String(format!("{}", e))))
            .unwrap();
        }
        _ => {}
      }
    });
  })?;

  let started_event = server.events.wait_until(ServerEvent::Started);
  let terminated_event = server.events.wait_until(ServerEvent::Terminated);
  tokio::select! {
    v = started_event => {
        // Note(sagar): we need to update address/port since the server uses
        // a random port by default
        let metadata: ServerStarted = serde_json::from_value(v?)
        .unwrap();

        let handle = WorkspaceServerHandle {
          workspace: server.workspace,
          address: metadata.address,
          port: metadata.port,
          handle: server.handle,
          events: server.events,
        };
        return Ok(handle)
    },
    e = terminated_event => {
      bail!("Error starting workspace server: {}", e?.as_str().unwrap_or("Unknown error"));
    },
  }
}

impl WorkspaceServer {
  async fn start_all(&mut self) -> Result<()> {
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

    info!(
      "Workspace [name={}] server started...",
      self.workspace.config.name
    );

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
          Err(e) => error!("JS runtime terminated with error: {:?}", e),
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
      // TODO(sagar): disabled this when running deployed workspace
      enable_console: true,
      transpile: true,
      enable_build_tools: true,
      // TODO(sagar): file permissions is required to server static files
      // move the fileserver to rust so that file permission isn't ncessary
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: self.workspace.dir.clone(),
          allowed_read_paths: HashSet::from_iter(vec![
            // Note(sagar): only give read access to workspace directory
            "./".to_owned(),
          ]),
          ..Default::default()
        }),
        ..Default::default()
      },
      extensions: vec![super::ext::init(Rc::new(RefCell::new(rx)))],
      heap_limits: self.workspace.heap_limits,
      ..Default::default()
    })?;

    // In debug mode, load the module dynamically during runtime but
    // in release mode, include the js code in the binary
    // let workspace_server_code = if cfg!(debug_assertions) {
    #[cfg(debug_assertions)]
    let workspace_server_code = std::fs::read_to_string(
      Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../js/packages/workspace-server/dist/index.js"),
    )?;
    #[cfg(not(debug_assertions))]
    let workspace_server_code =
      include_str!("../../../../js/packages/workspace-server/dist/index.js")
        .to_owned();

    let entry_file = self.workspace.entry_file();

    // Note(sagar): set the workspace server's module file in the
    // same directory as the workspace entry file so that we can
    // resolve files for the workspace using `import.meta.resolve`
    let virtual_workspace_server_file = &Url::parse(&format!(
      "file://{}/workspace-server.virutal.js",
      entry_file
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(anyhow!("unable to get workspace directory path"))?
    ))?;

    // Note(sagar): preload this module so that it can be used later
    runtime
      .execute_side_module(
        virtual_workspace_server_file,
        Some(workspace_server_code),
      )
      .await?;

    let entry_file = entry_file
      .to_str()
      .ok_or(anyhow!("Unable to get workspace entry file"))?;
    let receiver = runtime
      .execute_main_module_code(
        &Url::parse("file://arena/workspace-server/init")?,
        &format!(
          r#"
            import {{ serve }} from "{}";

            // Note(sagar): need to dynamically load the entry-server.tsx or
            // whatever the entry file is for the workspace so that it's
            // transpiled properly

            import("file://{}").then(async ({{ default: m }}) => {{
              serve(m);
            }});
          "#,
          virtual_workspace_server_file, entry_file
        ),
      )
      .await?;

    tokio::spawn(receiver);

    Ok(runtime)
  }
}
