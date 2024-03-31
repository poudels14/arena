use std::fs::File;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use futures::TryFutureExt;
use tar::Archive;
use tempdir::TempDir;

pub async fn download_and_extract_tgz(
  url: &str,
  destination: &Path,
) -> Result<()> {
  let dir = TempDir::new("portal-downloader")?;
  let file = dir.path().join("tmp-tar-download.tgz");

  download_from_url(url, &file)
    .map_err(|e| anyhow!("Error downloading: {:?}", e))
    .await?;

  let tar_gz = File::open(file)?;
  let tar = GzDecoder::new(tar_gz);
  let mut archive = Archive::new(tar);
  archive.unpack(destination)?;
  Ok(())
}

pub async fn download_from_url(url: &str, destination: &Path) -> Result<()> {
  let content = reqwest::get(url).await?.bytes().await?;
  std::fs::write(destination, content).context("error writing to tmp file")?;
  Ok(())
}
