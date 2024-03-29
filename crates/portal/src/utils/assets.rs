use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;

use anyhow::Result;
use bytes::Bytes;
use tar::Archive;

use super::encryption::decrypt;

static FRONTEND_BUNDLE_TAR: &'static [u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/frontend-bundle.tar"));

pub struct PortalAppModules {
  assets: HashMap<String, Bytes>,
}

impl PortalAppModules {
  pub fn new() -> Self {
    let cursor = Cursor::<&'static [u8]>::new(FRONTEND_BUNDLE_TAR);
    let mut assets = HashMap::new();
    let mut archive = Archive::new(cursor);
    let mut encryption_key =
      env!("PORTAL_DESKTOP_ENC_KEY").as_bytes().to_owned();
    encryption_key.reverse();
    for file in archive.entries().unwrap() {
      let file = file.unwrap();

      let filepath =
        file.header().path().unwrap().to_string_lossy().to_string();
      let enc_bytes = file
        .bytes()
        .collect::<Result<Vec<u8>, std::io::Error>>()
        .unwrap();

      let content = decrypt(&encryption_key, filepath.as_bytes(), enc_bytes);
      assets.insert(filepath, Bytes::from(content));
    }

    Self { assets }
  }

  pub fn get_asset<'a>(&'a self, path: &str) -> Result<Option<Bytes>> {
    Ok(self.assets.get(path).cloned())
  }
}
