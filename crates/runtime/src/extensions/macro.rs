macro_rules! js_dist {
  ($a:expr) => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/js/dist", $a)
      );
    }

    crate::extensions::r#macro::include_source_code!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/js/dist",
      $a
    ))
  }};
  ($a:expr, "runtime") => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/js/dist", $a)
      );
    }

    // Always include the code in the binary if "runtime" flag is set
    let source = crate::extensions::SourceCode::Runtime(include_str!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/js/dist",
      $a
    )));

    source
  }};
}

macro_rules! include_source_code {
  ($file:expr $(,)?) => {{
    #[cfg(feature = "fs_rerun_if_changed")]
    {
      println!("cargo:rerun-if-changed={}", $file);
    }

    // Include the code as Snapshot if "include-in-binary" feature is ON
    #[cfg(feature = "include-in-binary")]
    let source = crate::extensions::SourceCode::Preserved(include_str!($file));

    // If the "include-in-binary" feature is off, dont need to include
    // the code unless "runtime" flag is ON, in which case, another macro
    // handles it
    #[cfg(not(feature = "include-in-binary"))]
    let source = crate::extensions::SourceCode::NotPreserved;

    source
  }};
}

pub(crate) use include_source_code;
pub(crate) use js_dist;
