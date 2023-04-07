use deno_core::{ExtensionFileSource, ExtensionFileSourceCode};

pub mod exts;
pub mod transpiler;

pub(crate) fn get_build_tools_modules() -> Vec<ExtensionFileSource> {
  vec![
    ExtensionFileSource {
      specifier: "@arena/buildtools".to_owned(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../js/arena-runtime/dist/buildtools.js"
      )),
    },
    ExtensionFileSource {
      specifier: "@arena/babel".to_owned(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../js/arena-runtime/dist/babel.js"
      )),
    },
    ExtensionFileSource {
      specifier: "@arena/rollup".to_owned(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../js/arena-runtime/dist/rollup.js"
      )),
    },
  ]
}
