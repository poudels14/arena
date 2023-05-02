use anyhow::Result;
use clap::{value_parser, Parser};
use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};
use jsruntime::permissions::{FileSystemPermissions, PermissionsContainer};
use jsruntime::{IsolatedRuntime, RuntimeConfig};
use std::collections::HashSet;
use url::Url;

#[derive(Parser, Debug)]
pub struct Command {
  /// Whether to minify client bundle. Server bundle isn't minified
  #[arg(short('m'), long, value_parser=value_parser!(bool), default_value_t = true)]
  pub minify: bool,
}

impl Command {
  #[tracing::instrument(skip_all)]
  pub async fn execute(&self) -> Result<()> {
    let cwd = std::env::current_dir()?;
    let cwd = cwd.to_str().unwrap();
    let mut runtime = IsolatedRuntime::new(RuntimeConfig {
      enable_console: true,
      enable_build_tools: true,
      enable_node_modules: true,
      permissions: PermissionsContainer {
        fs: Some(FileSystemPermissions {
          root: cwd.into(),
          allowed_read_paths: HashSet::from_iter(vec![
            // allow all files in current directory
            cwd.to_string(),
          ]),
          allowed_write_paths: HashSet::from_iter(vec![
            // allow all files in current directory
            cwd.to_string(),
          ]),
          ..Default::default()
        }),
        ..Default::default()
      },
      side_modules: vec![
        ExtensionFileSource {
          specifier: "@arena/workspace-server/builder".to_owned(),
          code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
            "../../../../js/packages/workspace-server/dist/builder.js"
          )),
        },
        ExtensionFileSource {
          specifier: "dagger/bundler".to_owned(),
          code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
            "bundler.js"
          )),
        },
      ],
      ..Default::default()
    })?;

    runtime
      .execute_main_module_code(
        &Url::parse("file://main").unwrap(),
        &format!(
          r#"
        import {{ bundle }} from "dagger/bundler";
        bundle({{
          minify: {0}
        }})
        "#,
          self.minify
        ),
      )
      .await?;
    runtime.run_event_loop().await
  }
}
