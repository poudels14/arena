use anyhow::Result;
use clap::Parser;
use jsruntime::{IsolatedRuntime, RuntimeConfig};

#[derive(Parser, Debug)]
pub struct Command {
  /// Code to execute
  code: String,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      enable_build_tools: false,
      transpile: false,
      ..Default::default()
    });

    let function = runtime.init_js_function(&self.code)?;
    function.execute(vec![])?;

    runtime.run_event_loop().await
  }
}
