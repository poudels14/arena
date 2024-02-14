use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use rayon::ThreadPoolBuilder;
use tokio::net::UnixListener;
use tokio::signal;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

#[cfg(unix)]
use tokio_stream::wrappers::UnixListenerStream;

mod fs;
mod grpc;
mod portal;
mod runtime;

use grpc::server::PythonRuntimeServer;
use runtime::server::RuntimeServer;
use tonic::transport::Server;

/// Python runtime cluster
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// UNIX socket file
  #[arg(long)]
  socket_file: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
  let subscriber = tracing_subscriber::registry()
    .with(
      tracing_subscriber::filter::EnvFilter::from_default_env()
        .add_directive(Directive::from_str("tokio=OFF").unwrap())
        .add_directive(Directive::from_str("hyper=OFF").unwrap()),
    )
    .with(
      HierarchicalLayer::default()
        .with_indent_amount(2)
        .with_thread_names(true),
    );
  tracing::subscriber::set_global_default(subscriber).unwrap();

  let args = Args::parse();

  // Since all python code execution is done in rayon threadpool,
  // the number of threads determine how many concurrent code exec
  // can be run
  // It's very likely that at max, one code is executed in a runtime at
  // at any time because a chat thread is serial. So, just limit the thread
  // pool
  ThreadPoolBuilder::new()
    .num_threads(2)
    .build_global()
    .unwrap();

  let path = PathBuf::from(args.socket_file);
  let _ = std::fs::remove_file(&path);
  std::fs::create_dir_all(path.parent().unwrap()).unwrap();

  let uds = UnixListener::bind(&path)?;
  let uds_stream = UnixListenerStream::new(uds);
  println!("Listening on {:?}", &path);

  Server::builder()
    .add_service(PythonRuntimeServer::new(RuntimeServer {}))
    .serve_with_incoming_shutdown(uds_stream, async {
      signal::ctrl_c().await.unwrap()
    })
    .await?;

  Ok(())
}
