use crate::ast::types::{ident, member_expr};
use swc_common::Mark;
use swc_ecma_ast::{Expr, MemberExpr, MemberProp};
use swc_ecma_visit::VisitMut;

/// this looks for unresolved variables and adds "props." prefix to them
/// so that data from props generator can be used by the server function
pub(crate) struct PropsSanitizer {
  pub unresolved_mark: Mark,
  pub unresolved_exprs: Vec<Expr>,
}

impl VisitMut for PropsSanitizer {
  fn visit_mut_member_expr(&mut self, node: &mut MemberExpr) {
    let n = node.clone();
    let mut node = node;
    loop {
      if node.obj.is_member() {
        node = node.obj.as_mut_member().unwrap();
        continue;
      }
      if let Some(i) = node.obj.as_ident() {
        if i.span.has_mark(self.unresolved_mark) {
          node.obj = Expr::Member(member_expr(
            Expr::Ident(ident("props")),
            MemberProp::Ident(i.to_owned()),
          ))
          .into();
          self.unresolved_exprs.push(Expr::Member(n));
          break;
        }
      }
      break;
    }
  }
}
