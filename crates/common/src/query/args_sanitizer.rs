use crate::ast::types::{assign_object_pat_prop, ident, object_pat, param};
use swc_ecma_ast::ExportDefaultDecl;
use swc_ecma_ast::Pat;
use swc_ecma_visit::VisitMut;

/// this sanitizes the argument of the default exported function
/// to ensure that `{ env, props, ... }` variables are "injected"
/// in the server function context
pub(crate) struct ArgsSanitizer {}

impl VisitMut for ArgsSanitizer {
  fn visit_mut_export_default_decl(&mut self, node: &mut ExportDefaultDecl) {
    if let Some(expr) = node.decl.as_mut_fn_expr() {
      expr.function.params = vec![param(Pat::Object(object_pat(vec![
        assign_object_pat_prop(ident("env")),
        assign_object_pat_prop(ident("props")),
      ])))];
    }
  }
}
