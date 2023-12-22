use super::r#macro::js_dist;
use super::BuiltinExtension;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    None,
    vec![("@arena/runtime/bundler", js_dist!("/bundler.js"))],
  )
}
