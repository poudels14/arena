use anyhow::Result;
use clap::Parser;
use jsruntime::{IsolatedRuntime, RuntimeOptions};

#[derive(Parser, Debug)]
pub struct Command {
  /// Code to execute
  code: String,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let mut runtime = IsolatedRuntime::new(RuntimeOptions {
      enable_console: true,
      transpile: false,
      ..Default::default()
    })?;

    let function = runtime.init_js_function(&self.code, None)?;
    let result = function.execute(vec![])?.unwrap().get_value_async().await?;
    println!("{:?}", result);

    runtime.run_event_loop().await
  }
}
