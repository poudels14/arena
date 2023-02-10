use anyhow::Result;
use clap::Parser;
use deno_core::resolve_url_or_path;
use jsruntime::{IsolatedRuntime, RuntimeConfig};

#[derive(Parser, Debug)]
pub struct Command {
  /// File to execute
  file: String,

  /// Whether to auto-transpile code; default true
  #[arg(short, long)]
  disable_transpile: bool,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      transpile: !self.disable_transpile,
      ..Default::default()
    });

    let main_module = resolve_url_or_path(&self.file)?;
    runtime.execute_main_module(&main_module).await?;

    runtime.run_event_loop().await
  }
}
