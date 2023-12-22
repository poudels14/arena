use super::r#macro::js_dist;
use super::BuiltinExtension;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    None,
    vec![("@arena/runtime/filerouter", js_dist!("/filerouter.js"))],
  )
}
