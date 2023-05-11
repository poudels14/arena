use anyhow::Result;
use common::config::ArenaConfig;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;
use url::Url;
mod events;
mod ext;
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
      BuiltinModule::HttpServer("0.0.0.0", 8002),
      BuiltinModule::CustomRuntimeModule(
        "@arena/runtime/dqs",
        include_str!("../../../js/arena-runtime/dist/dqs.js"),
      ),
    ]),
    transpile: false,
    extensions: vec![crate::ext::init()],
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
            import { WorkspaceServer } from "builtin:///@arena/runtime/dqs";
            serve({
              async fetch(req) {
                const s = await WorkspaceServer.start('workspace_1');
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
