use super::BuiltinExtension;
use crate::extensions::r#macro::js_dist;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension::new(
    None,
    vec![("@arena/runtime/babel", js_dist!("/babel.js"))],
  )
}
