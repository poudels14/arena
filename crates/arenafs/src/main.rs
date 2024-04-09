use std::sync::{Arc, Mutex};

use clap::Parser;
use fuser::MountOption;

use crate::backend::postgres::PostgresBackend;
use crate::fs::{FileSystem, FilesCache};

mod backend;
mod error;
mod fs;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
  /// Database url to use
  /// Either set DATABASE_URL env variable or pass it as this arg
  #[arg(long, short)]
  db_url: Option<String>,

  #[arg(long)]
  auto_unmount: bool,
  /// The dir to mount the filesystem to
  mount_point: String,
}

#[tokio::main]
async fn main() {
  let args = Args::parse();
  env_logger::init();
  let db_url = args.db_url.unwrap_or_else(|| {
    std::env::var("DATABASE_URL")
      .expect("either pass `db_url` arg or set DATABASE_URL env variable")
  });

  let backend = PostgresBackend::init(&db_url, "files", true).await.unwrap();
  let filesystem = FileSystem::with_backend(
    fs::Options {
      root_id: None,
      user_id: 1000,
      group_id: 1000,
    },
    Arc::new(Mutex::new(FilesCache::new())),
    Arc::new(backend),
  )
  .await
  .unwrap();

  let mut options =
    vec![MountOption::RW, MountOption::FSName("arenafs".to_string())];
  if args.auto_unmount {
    options.push(MountOption::AutoUnmount);
  }
  options.push(MountOption::Suid);
  filesystem.mount(&args.mount_point, &options).unwrap();
}
