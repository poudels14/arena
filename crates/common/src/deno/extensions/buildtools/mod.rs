use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};

pub fn get_modules_for_snapshotting() -> Vec<ExtensionFileSource> {
  vec![
    ExtensionFileSource {
      specifier: "@arena/babel".to_owned(),
      code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/babel.js".into(),
      ),
    },
    ExtensionFileSource {
      specifier: "@arena/rollup".to_owned(),
      code: ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
        "../../js/arena-runtime/dist/rollup.js".into(),
      ),
    },
  ]
}
