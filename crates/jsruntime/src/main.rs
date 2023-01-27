mod buildtools;
mod core;
mod permissions;

use crate::core::{IsolatedRuntime, RuntimeConfig};
use deno_core::resolve_url_or_path;

#[tokio::main]
async fn main() {
  let mut runtime = IsolatedRuntime::new(RuntimeConfig {
    enable_console: true,
    enable_build_tools: true,
    ..Default::default()
  });

  let args: Vec<String> = std::env::args().collect();
  if args.len() > 1 {
    let main_module = resolve_url_or_path(&args[1]).unwrap();
    runtime.execute_main_module(&main_module).await.unwrap();
    runtime.run_event_loop().await.unwrap();
  }
}
