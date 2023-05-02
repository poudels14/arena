use super::events::{ServerEvent, ServerEvents, ServerStarted};
use super::ext::ResponseSender;
use super::http::HttpRequest;
use crate::server::{
  ClientRequest, ServerHandle, ServerOptions, ServerRequest,
  WorkspaceServerHandle,
};
use crate::{Workspace, WorkspaceConfig};
use anyhow::{anyhow, bail, Result};
use common::deno::extensions;
use common::deno::permissions::{FileSystemPermissions, PermissionsContainer};
use deno_core::{
  op, Extension, ExtensionFileSource, ExtensionFileSourceCode, OpState,
};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashSet;
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
  /// Whether to run the server in dev mode
  pub dev_mode: bool,

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
    dev_mode: options.dev_mode,
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
    Err(e) = terminated_event => bail!("Error starting workspace server: {:?}", e),
  }
}

impl WorkspaceServer {
  async fn start_all(&mut self) -> Result<()> {
    let (tx, rx) = mpsc::channel::<(HttpRequest, ResponseSender)>(100);
    self.vm_serice = Some(VmService { sender: tx });
    let js_server = self.start_workspace_js_server(rx);

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
      c = js_server => {
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
  ) -> Result<()> {
    // TODO(sagar): only give read access to workspace directory
    let mut allowed_read_paths = vec![];
    if let Some(dir) = self.workspace.dir.to_str() {
      allowed_read_paths.push(dir.to_string());
    }

    let workspace_config = self.workspace.config.clone();
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      // TODO(sagar): disabled this when running deployed workspace
      enable_console: true,
      transpile: self.dev_mode,
      enable_build_tools: self.dev_mode,
      enable_node_modules: true,
      // TODO(sagar): file permissions is required to server static files
      // move the fileserver to rust so that file permission isn't ncessary
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: self.workspace.dir.clone(),
          allowed_read_paths: HashSet::from_iter(allowed_read_paths),
          ..Default::default()
        }),
        ..Default::default()
      },
      extensions: vec![
        super::ext::init(Rc::new(RefCell::new(rx))),
        Extension::builder("arena/workspace-server/config")
          .ops(vec![op_load_workspace_config::decl()])
          .state(move |state| {
            state.put::<WorkspaceConfig>(workspace_config.clone());
          })
          .js(vec![ExtensionFileSource {
            specifier: "init".to_owned(),
            code: ExtensionFileSourceCode::IncludedInBinary(
              r#"
              Object.assign(globalThis.Arena, {
                Workspace: {
                  config: Arena.core.ops.op_load_workspace_config()
                }
              });
              "#,
            ),
          }])
          .build(),
        extensions::postgres::init(),
      ],
      side_modules: vec![
        vec![ExtensionFileSource {
          specifier: "@arena/workspace-server".to_owned(),
          code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
            "../../../../js/packages/workspace-server/dist/server.js"
          )),
        }],
        extensions::postgres::get_modules_for_snapshotting(),
      ]
      .concat(),
      heap_limits: self.workspace.heap_limits,
      ..Default::default()
    })?;

    let server_entry = self.workspace.server_entry();
    let server_entry = server_entry
      .to_str()
      .ok_or(anyhow!("Unable to get workspace entry file"))?;

    runtime
      .execute_main_module_code(
        &Url::parse("file:///arena/workspace-server/init")?,
        &format!(
          r#"
            import {{ serve }} from "@arena/workspace-server";

            // Note(sagar): need to dynamically load the entry-server.tsx or
            // whatever the entry file is for the workspace so that it's
            // transpiled properly

            await import("file://{}").then(async ({{ default: m }}) => {{
              serve(m, {{
                serveFiles: {}
              }});
            }});
          "#,
          server_entry, self.dev_mode
        ),
      )
      .await?;

    runtime.run_event_loop().await?;
    Ok(())
  }
}

#[op]
pub fn op_load_workspace_config(state: &mut OpState) -> Result<Value> {
  let config = state.borrow_mut::<WorkspaceConfig>();
  Ok(json!(config))
}
