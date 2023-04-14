use common::deno::extensions::node::duplicate_modules;
use deno_core::ExtensionFileSource;
use deno_core::ExtensionFileSourceCode;

pub fn get_runtime_modules() -> Vec<ExtensionFileSource> {
  vec![
    // Note(sagar): this exposes node modules, so should be loaded
    // only when node-modules are enabled
    vec![ExtensionFileSource {
      specifier: "node/setup".to_owned(),
      code: ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "./node.js"
      )),
    }],
    // Note(sagar): for some reason, 3+ modules have to be loaded during
    // runtime to prevent segfault. My guess is race condition is causing
    // the segfault
    // TODO(sagar): this somehow fixs segfault; makes no fucking sense
    duplicate_modules(
      vec!["_rand2_", "_rand3_", "_rand4_"],
      ExtensionFileSourceCode::IncludedInBinary(include_str!(
        "../../../../js/arena-runtime/dist/node/util.js"
      )),
    ),
  ]
  .concat()
}
