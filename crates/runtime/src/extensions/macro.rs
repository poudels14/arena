macro_rules! js_dist {
  ($a:expr) => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/js/dist", $a)
      );
    }

    crate::extensions::r#macro::source_code!(include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/js/dist",
      $a
    )))
  }};
  ($a:expr, "runtime") => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/js/dist", $a)
      );
    }

    // If the snapshot-build-tools feature is off, include bytes in the
    // binary
    #[cfg(not(feature = "snapshot-build-tools"))]
    let source = crate::extensions::SourceCode::Runtime(include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/js/dist",
      $a
    )));

    source
  }};
}

macro_rules! source_code {
  ($code:expr) => {{
    // Include the code as Snapshot if build tools feature is ON
    #[cfg(feature = "snapshot-build-tools")]
    let source = crate::extensions::SourceCode::Snapshot($code);

    // If the snapshot-build-tools feature is off, dont need to include
    // the code unless "runtime" flag is ON, in which case, another macro
    // handles it
    #[cfg(not(feature = "snapshot-build-tools"))]
    let source = crate::extensions::SourceCode::Runtime("");

    source
  }};
}

pub(crate) use js_dist;
pub(crate) use source_code;
