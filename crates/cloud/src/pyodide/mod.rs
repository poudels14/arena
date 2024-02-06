use std::io::Read;
use std::path::PathBuf;
use std::sync::Once;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use runtime::deno::core::{op2, OpState};
use tar::Archive;

pub static BINARIES_ARCHIVE: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/pyodide.tar.gz"));

static UNARCHIVE: Once = Once::new();
static mut TEMP_DIR: Option<PathBuf> = None;

#[tracing::instrument(skip(_state), level = "trace")]
#[op2]
#[string]
pub fn op_cloud_pyodide_load_text_file(
  _state: &mut OpState,
  #[string] path: String,
) -> Result<String> {
  let tmp_dir = setup_files_once()?;
  let filename = path.replace("builtin://@arena/cloud/pyodide/", "");
  let tmp_path = tmp_dir.join(&filename);
  std::fs::read_to_string(tmp_path).context(format!("reading {:?}", filename))
}

#[tracing::instrument(skip(_state), level = "trace")]
#[op2]
#[arraybuffer]
pub fn op_cloud_pyoddide_load_binary(
  _state: &mut OpState,
  #[string] path: String,
) -> Result<Vec<u8>> {
  let tmp_dir = setup_files_once()?;
  let filename = path.replace("builtin://@arena/cloud/pyodide/", "");
  let tmp_path = tmp_dir.join(&filename);

  let bytes =
    std::fs::read(tmp_path).context(format!("reading {:?}", filename))?;
  Ok(bytes)
}

fn setup_files_once() -> Result<PathBuf> {
  UNARCHIVE.call_once(|| unsafe {
    let now = Instant::now();
    TEMP_DIR = tempfile::tempdir().map(|dir| dir.into_path()).ok();
    if TEMP_DIR.is_none() {
      return;
    }

    let tmpdir = TEMP_DIR.clone().unwrap();
    let mut archive = BINARIES_ARCHIVE;
    let mut archive = Archive::new(&mut archive);

    for file in archive.entries().unwrap() {
      let mut file = file.unwrap();
      let mut buf = vec![];
      file.read_to_end(&mut buf).unwrap();
      std::fs::write(
        tmpdir.join(file.header().path().unwrap().to_str().unwrap()),
        buf,
      )
      .unwrap();
    }
    tracing::debug!("cloud py load time = {:?}", now.elapsed().as_millis());
  });
  unsafe {
    TEMP_DIR
      .clone()
      .ok_or(anyhow!("Error initializing setup files"))
  }
}
