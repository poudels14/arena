use std::collections::HashMap;
use std::io::Cursor;
use std::io::Read;

use anyhow::Result;
use bytes::Bytes;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use tar::Archive;

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

fn decrypt(key: &[u8], nonce: &[u8], mut encrypted_data: Vec<u8>) -> Vec<u8> {
  let unbound_key =
    UnboundKey::new(&AES_256_GCM, key).expect("encryption error");

  let mut nonce_slice = [0u8; 12];
  nonce_slice.copy_from_slice(&nonce[0..12]);
  let nonce = Nonce::assume_unique_for_key(nonce_slice);
  let aad = Aad::empty();

  let s_key = LessSafeKey::new(unbound_key);
  let decrypted_data = s_key
    .open_in_place(nonce, aad, &mut encrypted_data)
    .expect("encryption error");

  decrypted_data[..decrypted_data.len() - AES_256_GCM.tag_len()].to_vec()
}
