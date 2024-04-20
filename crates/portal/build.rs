use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

extern crate napi_build;

use ring::aead;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};
use tar::Builder;
use tar::Header;
use walkdir::WalkDir;

fn main() {
  println!(
    "cargo:rustc-env=TARGET={}",
    std::env::var("TARGET").unwrap()
  );
  let key_len = AES_256_GCM.key_len();
  let encryption_key = nanoid::nanoid!(key_len);
  println!("cargo:rustc-env=PORTAL_DESKTOP_ENC_KEY={}", &encryption_key);
  let mut encryption_key = encryption_key.as_bytes().to_owned();
  encryption_key.reverse();
  let packages = vec![
    (
      "workspace-desktop",
      "0.1.7",
      "PORTAL_DESKTOP_WORKSPACE_VERSION",
      #[cfg(debug_assertions)]
      "../../js/workspace-desktop/dist/workspace-desktop",
      #[cfg(not(debug_assertions))]
      "../../app-bundles/apps/workspace-desktop",
    ),
    (
      "atlasai",
      "0.1.6",
      "PORTAL_DESKTOP_ATLAS_VERSION",
      #[cfg(debug_assertions)]
      "../../js/apps/atlasai/dist/atlasai",
      #[cfg(not(debug_assertions))]
      "../../app-bundles/apps/atlasai",
    ),
    (
      "portal-drive",
      "0.1.5",
      "PORTAL_DESKTOP_DRIVE_VERSION",
      #[cfg(debug_assertions)]
      "../../js/apps/drive/dist/portal-drive",
      #[cfg(not(debug_assertions))]
      "../../app-bundles/apps/portal-drive",
    ),
  ];

  let archive_file = File::create(format!(
    "{}/frontend-bundle.tar",
    std::env::var_os("OUT_DIR")
      .unwrap()
      .to_str()
      .expect("getting OUT_DIR")
  ))
  .expect("Error creating archive file");
  let mut archive = Builder::new(archive_file);
  for package in packages {
    println!(
      "cargo:warning=Adding [{}@{}] to bundle: {}/{}",
      package.0, package.1, package.3, package.1
    );
    println!("cargo:rustc-env={}={}", package.2, package.1);

    let base = PathBuf::from(package.3);
    add_directory_to_archive(
      &mut archive,
      &encryption_key,
      package.0,
      package.1,
      base.join(package.1),
    );
  }
  napi_build::setup();
}

fn add_directory_to_archive(
  archive: &mut Builder<File>,
  encryption_key: &[u8],
  package: &str,
  version: &str,
  base: PathBuf,
) {
  if !base.exists() {
    panic!("Bundle directory doesn't exist: {:?}", base);
  }
  for entry in WalkDir::new(&base)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| !e.file_type().is_dir())
  {
    let path =
      pathdiff::diff_paths::<&Path, &PathBuf>(entry.path(), &base).unwrap();
    let filepath =
      format!("{}/{}/{}", package, version, path.to_str().unwrap());
    let mut content = vec![];
    File::open(entry.path())
      .unwrap()
      .read_to_end(&mut content)
      .expect("error reading file content");

    let mut header = Header::new_gnu();
    header
      .set_path(&filepath)
      .expect("error setting header path");

    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&filepath.as_bytes()[0..12]);
    let enc_content = encrypt(encryption_key, nonce, &content);

    header.set_size(enc_content.len() as u64);
    header.set_cksum();

    archive.append(&header, Cursor::new(enc_content)).unwrap();
    println!("cargo:rerun-if-changed={}", entry.path().display());
  }
}

fn encrypt(key: &[u8], nonce: [u8; aead::NONCE_LEN], data: &[u8]) -> Vec<u8> {
  let key = UnboundKey::new(&AES_256_GCM, key).expect("error creating key");
  let nonce_sequence = Nonce::assume_unique_for_key(nonce);
  let aad = Aad::empty();

  let mut in_out = data.to_vec();
  in_out.extend_from_slice(&vec![0; AES_256_GCM.tag_len()]);
  let s_key = LessSafeKey::new(key);

  s_key
    .seal_in_place_append_tag(nonce_sequence, aad, &mut in_out)
    .expect("error encrypting");
  in_out
}
