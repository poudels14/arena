use anyhow::Result;
use common::arena::ArenaConfig;
use common::deno::extensions::server::HttpServerConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use std::rc::Rc;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;
use url::Url;
mod apps;
mod config;
mod db;
mod extension;
mod loaders;
mod server;
mod specifier;

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
  let mut runtime = IsolatedRuntime::new(RuntimeOptions {
    project_root: Some(project_root.clone()),
    config: Some(ArenaConfig::default()),
    enable_console: true,
    builtin_extensions: BuiltinExtensions::with_modules(vec![
      BuiltinModule::HttpServer(HttpServerConfig::Tcp {
        address: "0.0.0.0".to_owned(),
        port: 8002,
        serve_dir: None,
      }),
      BuiltinModule::Custom(Rc::new(crate::extension::extension)),
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
          import { serve } from "@arena/runtime/server";
          import { DqsCluster } from "@arena/runtime/dqs";
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
                server = await DqsCluster.startStreamServer(workspaceId);
                servers.set(workspaceId, server);
              }

              const [status, headers, body] = await server.pipeRequest({
                url: `http://0.0.0.0/${appId}/widget/${widgetId}/api/${field}`,
                method: "POST",
                headers: [[ "content-type", "application/json" ]],
                body: await req.json()
              });
              return new Response(body, {
                status
              });
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
