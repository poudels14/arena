mod auth;
mod error;
mod init;
mod io;
mod pgwire;
mod schema;
mod server;
mod system;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::Parser;
use init::InitCluster;
use log::LevelFilter;
use server::ServerOptions;

use signal_hook::consts::TERM_SIGNALS;
use signal_hook::iterator::exfiltrator::SignalOnly;
use signal_hook::iterator::SignalsInfo;
#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

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

  let _ = rt
    .block_on(async {
      let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
      let handle: JoinHandle<Result<(), anyhow::Error>> = tokio::spawn(async {
        match args.command {
          Commands::Init(cmd) => cmd.execute().await,
          Commands::Start(cmd) => cmd.execute(shutdown_rx).await,
        }
      });

      let term_now = Arc::new(AtomicBool::new(false));
      for sig in TERM_SIGNALS {
        signal_hook::flag::register_conditional_shutdown(
          *sig,
          1,
          Arc::clone(&term_now),
        )
        .unwrap();
        signal_hook::flag::register(*sig, Arc::clone(&term_now)).unwrap();
      }
      let mut signals = SignalsInfo::<SignalOnly>::new(TERM_SIGNALS).unwrap();

      for _ in &mut signals {
        shutdown_tx.send(()).unwrap();
        break;
      }

      handle.await
    })
    .unwrap();
}
