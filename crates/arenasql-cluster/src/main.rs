mod auth;
mod error;
mod init;
mod io;
mod pgwire;
mod schema;
mod server;

use anyhow::Error;
use clap::Parser;
use init::InitCluster;
use log::LevelFilter;
use server::ServerOptions;

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Arena DB cluster
#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// Number of threads to use
  #[clap(short, long)]
  threads: Option<usize>,

  #[command(subcommand)]
  command: Commands,
}

#[derive(clap::Subcommand, Debug, Clone)]
enum Commands {
  /// Initialize Arena DB cluster
  Init(InitCluster),

  /// Start Arena DB cluster server
  Start(ServerOptions),
}

fn main() {
  env_logger::Builder::new()
    .filter_level(LevelFilter::Info)
    .parse_default_env()
    .init();

  let args = Args::parse();
  let num_thread = args.threads.unwrap_or(num_cpus::get());

  let rt = tokio::runtime::Builder::new_multi_thread()
    .worker_threads(num_thread)
    .enable_all()
    .build()
    .unwrap();

  rt.block_on(async {
    match args.command {
      Commands::Init(cmd) => cmd.execute().await?,
      Commands::Start(cmd) => cmd.execute().await?,
    };
    Ok::<(), Error>(())
  })
  .unwrap();
}
