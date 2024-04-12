#[cfg(not(feature = "unix-socket"))]
use std::net::SocketAddr;
#[cfg(feature = "unix-socket")]
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use rayon::ThreadPoolBuilder;
#[cfg(feature = "unix-socket")]
use tokio::net::UnixListener;
use tokio::signal;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_tree::HierarchicalLayer;

#[cfg(feature = "unix-socket")]
use tokio_stream::wrappers::UnixListenerStream;

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
  #[cfg(feature = "unix-socket")]
  /// UNIX socket file
  #[arg(long)]
  socket_file: String,

  #[cfg(not(feature = "unix-socket"))]
  /// Port
  #[arg(long)]
  port: u16,
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

  let server =
    Server::builder().add_service(PythonRuntimeServer::new(RuntimeServer {}));

  #[cfg(feature = "unix-socket")]
  let server = {
    let path = PathBuf::from(args.socket_file);
    let _ = std::fs::remove_file(&path);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();

    let uds = UnixListener::bind(&path)?;
    let stream = UnixListenerStream::new(uds);
    println!("Listening on {:?}", &path);
    server.serve_with_incoming_shutdown(stream, async {
      signal::ctrl_c().await.unwrap()
    })
  };

  #[cfg(not(feature = "unix-socket"))]
  let server = {
    println!("Listening on port {:?}", &args.port);
    server.serve_with_shutdown(
      format!("0.0.0.0:{}", args.port).parse::<SocketAddr>()?,
      async { signal::ctrl_c().await.unwrap() },
    )
  };

  server.await?;
  Ok(())
}
