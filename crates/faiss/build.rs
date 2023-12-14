use std::fs::{self, DirEntry};
use std::path::Path;

use bindgen::builder;

fn main() {
  #[cfg(feature = "static")]
  static_link_faiss();
  generate_bindings();
}

fn generate_bindings() {
  let root = Path::new("./faiss/");
  let mut bldr = builder()
    .size_t_is_usize(true)
    .allowlist_function("faiss_.*")
    .allowlist_type("idx_t|Faiss.*")
    .opaque_type("FILE");

  let dirs = vec![
    root.join("c_api"),
    root.join("c_api/impl/"),
    root.join("c_api/utils/"),
  ];

  let files = dirs
    .iter()
    .map(|d| {
      println!("cargo:rerun-if-changed={}", d.to_str().unwrap());
      fs::read_dir(d).unwrap()
    })
    .into_iter()
    .flat_map(|d| d.into_iter())
    .collect::<Result<Vec<DirEntry>, std::io::Error>>()
    .unwrap();
  for f in files {
    if f.path().as_os_str().to_str().unwrap().ends_with("_c.h") {
      bldr = bldr.header(f.path().to_str().unwrap());
    }
  }
  let bindings = bldr.generate().unwrap();

  // Write the generated bindings to an output file.
  bindings.write_to_file("src/faiss_ffi/bindings.rs").unwrap();
}

// Credit: faiss_rs
#[cfg(feature = "static")]
fn static_link_faiss() {
  let mut cfg = cmake::Config::new("faiss");
  cfg
    .define("FAISS_ENABLE_C_API", "ON")
    .define("BUILD_SHARED_LIBS", "OFF")
    .define("CMAKE_BUILD_TYPE", "Release")
    .define(
      "FAISS_ENABLE_GPU",
      if cfg!(feature = "gpu") { "ON" } else { "OFF" },
    )
    .define("FAISS_ENABLE_PYTHON", "OFF")
    .define("BUILD_TESTING", "OFF")
    .very_verbose(true);
  let dst = cfg.build();
  let faiss_location = dst.join("lib");
  let faiss_c_location = dst.join("build/c_api");
  println!(
    "cargo:rustc-link-search=native={}",
    faiss_location.display()
  );
  println!(
    "cargo:rustc-link-search=native={}",
    faiss_c_location.display()
  );
  println!("cargo:rustc-link-lib=static=faiss_c");
  println!("cargo:rustc-link-lib=static=faiss");
  link_cxx();
  println!("cargo:rustc-link-lib=gomp");
  println!("cargo:rustc-link-lib=blas");
  println!("cargo:rustc-link-lib=lapack");
  // TODO
  // if cfg!(feature = "gpu") {
  //   let cuda_path = cuda_lib_path();
  //   println!("cargo:rustc-link-search=native={}/lib64", cuda_path);
  //   println!("cargo:rustc-link-lib=cudart");
  //   println!("cargo:rustc-link-lib=cublas");
  // }
}

// Credit: faiss_rs
#[cfg(feature = "static")]
fn link_cxx() {
  let cxx = match std::env::var("CXXSTDLIB") {
    Ok(s) if s.is_empty() => None,
    Ok(s) => Some(s),
    Err(_) => {
      let target = std::env::var("TARGET").unwrap();
      if target.contains("msvc") {
        None
      } else if target.contains("apple")
        | target.contains("freebsd")
        | target.contains("openbsd")
      {
        Some("c++".to_string())
      } else {
        Some("stdc++".to_string())
      }
    }
  };
  if let Some(cxx) = cxx {
    println!("cargo:rustc-link-lib={}", cxx);
  }
}
