use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use tar::Builder;
use walkdir::WalkDir;

fn main() {
  let packages = vec![
    (
      "workspace-desktop",
      "0.0.2",
      "PORTAL_DESKTOP_WORKSPACE_VERSION",
      "../../js/workspace-cluster/dist/workspace-desktop",
    ),
    (
      "atlasai",
      "0.0.2",
      "PORTAL_DESKTOP_ATLAS_VERSION",
      "../../js/workspace-cluster/dist/workspace-desktop",
    ),
    (
      "portal-drive",
      "0.0.2",
      "PORTAL_DESKTOP_DRIVE_VERSION",
      "../../js/workspace-cluster/dist/workspace-desktop",
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
      package.0,
      package.1,
      base.join(package.1),
    );
  }
}

fn add_directory_to_archive(
  archive: &mut Builder<File>,
  package: &str,
  version: &str,
  base: PathBuf,
) {
  for entry in WalkDir::new(&base)
    .into_iter()
    .filter_map(Result::ok)
    .filter(|e| !e.file_type().is_dir())
  {
    let path =
      pathdiff::diff_paths::<&Path, &PathBuf>(entry.path(), &base).unwrap();
    archive
      .append_file(
        format!("{}/{}/{}", package, version, path.to_str().unwrap()),
        &mut File::open(entry.path()).unwrap(),
      )
      .unwrap();
    println!("cargo:rerun-if-changed={}", entry.path().display());
  }
}
