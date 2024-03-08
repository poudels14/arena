use std::env;
use std::path::Path;
use std::rc::Rc;

use anyhow::Result;
use clap::Parser;
use common::required_env;
use runtime::buildtools::FileModuleLoader;
use runtime::config::ArenaConfig;
use runtime::extensions::BuiltinModule;
use runtime::resolver::FilePathResolver;
use runtime::utils::fs::has_file_in_file_tree;
use s3::creds::Credentials;
use tracing::info;

use crate::app::server::{self, ServerOptions};
use crate::utils::s3loader::{S3ModuleLoaderOptions, S3ModulerLoader};

#[derive(Parser, Debug)]
pub struct Command {
  /// App id
  /// This must be set for ACL checker to work
  #[arg(long)]
  pub app_id: Option<String>,

  /// If true, headers won't be filtered and all headers
  // will be passed through the proxy
  #[arg(long)]
  pub allow_headers: Option<bool>,

  /// Port to listen to
  #[arg(short, long, default_value_t = 8000)]
  pub port: u16,

  /// Directory of a workspace to serve; defaults to `${pwd}`
  #[arg(short, long)]
  pub dir: Option<String>,

  /// Heap limit hint
  #[arg(long)]
  heap_limit_mb: Option<usize>,

  /// Load main module from the given S3 bucket
  /// The following env variables must be set
  ///   - S3_ENDPOINT
  ///   - S3_ACCESS_KEY
  ///   - S3_SECRET_KEY
  #[arg(long)]
  s3bucket: Option<String>,

  entry: String,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let cwd = env::current_dir()?;
    let app_dir = self
      .dir
      .as_ref()
      .map_or_else(
        || has_file_in_file_tree(Some(&cwd), "package.json"),
        |p| Some(Path::new(&p).to_path_buf()),
      )
      .unwrap_or_else(|| cwd.clone())
      .canonicalize()
      .expect("Error canonicalizing app dir");

    let server_entry = match self.s3bucket {
      Some(_) => self.entry.clone(),
      _ => {
        let entry = app_dir.join(&self.entry);
        entry
          .to_str()
          .expect("Error getting server entry path as str")
          .to_owned()
      }
    };

    let server_options = ServerOptions {
      app_id: self.app_id.clone(),
      allow_headers: self.allow_headers.clone(),
      address: "0.0.0.0".to_owned(),
      port: self.port,
      root_dir: app_dir.clone(),
      heap_limit_mb: self.heap_limit_mb,
      builtin_modules: vec![
        BuiltinModule::Env,
        BuiltinModule::Fs,
        BuiltinModule::Node(None),
        BuiltinModule::Postgres,
        BuiltinModule::Cloudflare,
      ],
      module_loader: if let Some(bucket) = &self.s3bucket {
        let endpoint = required_env!("S3_ENDPOINT");
        let access_key = required_env!("S3_ACCESS_KEY");
        let access_secret = required_env!("S3_ACCESS_SECRET");
        Some(Rc::new(S3ModulerLoader::new(S3ModuleLoaderOptions {
          with_path_style: true,
          bucket: bucket.to_owned(),
          endpoint,
          credentials: Credentials {
            access_key: Some(access_key),
            secret_key: Some(access_secret),
            security_token: None,
            session_token: None,
            expiration: None,
          },
        })))
      } else {
        let config = ArenaConfig::load(&cwd.join(&app_dir).canonicalize()?)?;
        Some(Rc::new(FileModuleLoader::new(
          Rc::new(FilePathResolver::new(
            app_dir.clone(),
            config
              .server
              .javascript
              .clone()
              .and_then(|j| j.resolve)
              .unwrap_or_default(),
          )),
          None,
        )))
      },
    };

    info!(
      "Startin dev server at {}:{}",
      server_options.address, server_options.port
    );
    server::start_js_server(
      server_options,
      &format!(
        r#"
          import {{ serve }} from "@arena/runtime/server";
          // Note(sagar): need to dynamically load the entry-server.tsx or
          // whatever the entry file is for the workspace so that it's
          // transpiled properly

          // Note(sagar): since this is running in server, set SSR = true
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

    Ok(())
  }
}
