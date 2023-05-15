use anyhow::Result;
use common::config::ArenaConfig;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;
use url::Url;
mod extension;
mod loaders;
mod runtime;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(tracing_subscriber::filter::EnvFilter::from_default_env())
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    );
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let project_root = ArenaConfig::find_project_root()?;
  let mut runtime = IsolatedRuntime::new(RuntimeConfig {
    project_root: Some(project_root.clone()),
    config: Some(ArenaConfig::default()),
    enable_console: true,
    builtin_extensions: BuiltinExtensions::with_modules(vec![
      BuiltinModule::Node,
      BuiltinModule::Transpiler,
      BuiltinModule::Resolver(project_root),
      BuiltinModule::HttpServer(HttpServerConfig::Tcp(
        "0.0.0.0".to_owned(),
        8002,
      )),
      BuiltinModule::Custom(crate::extension::extension),
      BuiltinModule::CustomRuntimeModule(
        "@arena/runtime/dqs",
        include_str!("../../../js/arena-runtime/dist/dqs.js"),
      ),
    ]),
    ..Default::default()
  })?;

  let local = tokio::task::LocalSet::new();
  local
    .run_until(async move {
      runtime
        .execute_main_module_code(
          &Url::parse("file:///main").unwrap(),
          r#"
          console.log('loading server module');
          import { serve } from "@arena/runtime/server";
          import { DqsServer } from "builtin:///@arena/runtime/dqs";
          const servers = new Map();
          serve({
            async fetch(req) {
              const url = new URL(req.url);
              if (url.pathname.startsWith("/terminate/")) {
                await Arena.core.opAsync(
                  'op_dqs_terminate_server',
                  parseInt(url.pathname.substr("/terminate/".length)));
                return "OK";
              }

              const workspaceId = 'workspace_1';
              let server = servers.get(workspaceId);
              if (!server || !server.isAlive()) {
                console.log("starting server for workspace =", workspaceId);
                server = await DqsServer.startStreamServer(workspaceId);
                servers.set(workspaceId, server);
              }
              const res = await server.pipeRequest(new Request(
                "http://0.0.0.0/execSql", {
                  headers: [],
                }
              ));
              console.log("BODY =", String.fromCharCode.apply(null, res[2]));
              return new Response('workspace server started');
            }
          })
          "#,
        )
        .await
        .unwrap();

      runtime.run_event_loop().await.unwrap();
    })
    .await;

  Ok(())
}
