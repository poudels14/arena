use super::events::{ServerEvent, ServerEvents, ServerStarted};
use crate::server::{
  ClientRequest, ServerHandle, ServerOptions, ServerRequest,
  WorkspaceServerHandle,
};
use crate::Workspace;
use anyhow::{anyhow, bail, Result};
use common::arena::ArenaConfig;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use common::deno::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::permissions::FetchPermissions;
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::thread;
use tracing::{debug, error, info};
use url::Url;

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
    let js_server = self.start_workspace_js_server();

    let command_listener = self.listen_to_admin_commands();

    info!(
      "Workspace [name={}] server started...",
      self.workspace.config.name
    );

    tokio::select! {
      c = command_listener => {
        debug!("admin command listener terminated: {:?}", c);
      },
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

  async fn start_workspace_js_server(&self) -> Result<()> {
    let mut builtin_modules = vec![
      BuiltinModule::Fs,
      BuiltinModule::Env,
      BuiltinModule::Node,
      BuiltinModule::Postgres,
      BuiltinModule::HttpServer(HttpServerConfig::Tcp {
        address: self.address.clone(),
        port: self.port,
        serve_dir: if self.dev_mode {
          None
        } else {
          Some(self.workspace.dir.clone())
        },
      }),
      BuiltinModule::Custom(cloud::extension::extension),
    ];

    if self.dev_mode {
      builtin_modules.extend(vec![
        BuiltinModule::Resolver(self.workspace.project_root()),
        BuiltinModule::Transpiler,
        BuiltinModule::FileRouter,
      ])
    }

    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      // TODO(sagar): disabled this when running deployed workspace
      project_root: Some(self.workspace.project_root()),
      config: ArenaConfig::find_in_path_hierachy(),
      enable_console: true,
      builtin_extensions: BuiltinExtensions::with_modules(builtin_modules),
      transpile: self.dev_mode,
      // TODO(sagar): file permissions is required to server static files
      // move the fileserver to rust so that file permission isn't ncessary
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: self.workspace.dir.clone(),
          // Note(sp): only give read access to workspace directory
          allowed_read_paths: HashSet::from_iter(vec![".".to_owned()]),
          ..Default::default()
        }),
        net: Some(FetchPermissions {
          restricted_urls: Some(Default::default()),
          ..Default::default()
        }),
        ..Default::default()
      },
      heap_limits: self.workspace.heap_limits,
      ..Default::default()
    })?;

    let server_entry = self.workspace.server_entry();
    let server_entry = server_entry
      .to_str()
      .ok_or(anyhow!("Unable to get workspace entry file"))?;

    let local = tokio::task::LocalSet::new();
    local
      .run_until(async move {
        runtime
          .execute_main_module_code(
            &Url::parse("file:///arena/workspace-server/init")?,
            &format!(
              r#"
                import {{ serve }} from "@arena/runtime/server";

                // Note(sagar): need to dynamically load the entry-server.tsx or
                // whatever the entry file is for the workspace so that it's
                // transpiled properly

                // Note(sagar): since this is running in server, set SSR = true
                Arena.env.SSR = "true";
                process.env.SSR = "true";
                await import("file://{}").then(async ({{ default: m }}) => {{
                  serve(m);
                  console.log("[Workspace Server]: Listening to connections...");
                }});
              "#,
              server_entry
            ),
          )
          .await?;

        runtime.run_event_loop().await?;

        Ok(())
      })
      .await
  }
}
