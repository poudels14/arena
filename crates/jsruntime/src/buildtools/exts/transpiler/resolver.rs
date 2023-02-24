use super::Transpiler;
use crate::buildtools::exts::resolver::resolve;
use std::borrow::Borrow;
use std::rc::Rc;
use swc_ecma_ast::Str;
use swc_ecma_ast::{ExportAll, ImportDecl};
use swc_ecma_visit::{as_folder, Fold, VisitMut};

pub(crate) struct Resolver {
  transpiler: Rc<Transpiler>,

  referrer: String,
}

pub(crate) fn resolver(
  transpiler: Rc<Transpiler>,
  referrer: &str,
) -> impl Fold + VisitMut {
  as_folder(Resolver {
    transpiler,
    referrer: referrer.to_string(),
  })
}

impl VisitMut for Resolver {
  fn visit_mut_import_decl(&mut self, node: &mut ImportDecl) {
    let resolved_path = resolve(
      self.transpiler.resolver.borrow(),
      &self.transpiler.root,
      &self.referrer,
      node.src.value.as_ref(),
    )
    .unwrap();

    let resolved_path_str = format!("\"{}\"", resolved_path);
    // TODO(sagar): fix source map?
    node.src = Box::new(Str {
      raw: Some(resolved_path_str.clone().into()),
      span: node.src.span,
      value: resolved_path_str.into(),
    });
  }

  fn visit_mut_export_all(&mut self, node: &mut ExportAll) {
    let src = &node.src;
    let resolved_path = resolve(
      self.transpiler.resolver.borrow(),
      &self.transpiler.root,
      &self.referrer,
      src.value.as_ref(),
    )
    .unwrap();

    let resolved_path_str = format!("\"{}\"", resolved_path);
    // TODO(sagar): fix source map?
    node.src = Box::new(Str {
      raw: Some(resolved_path_str.clone().into()),
      span: src.span,
      value: resolved_path_str.into(),
    });
  }

  fn visit_mut_named_export(&mut self, node: &mut swc_ecma_ast::NamedExport) {
    if let Some(src) = &node.src {
      let resolved_path = resolve(
        self.transpiler.resolver.borrow(),
        &self.transpiler.root,
        &self.referrer,
        src.value.as_ref(),
      )
      .unwrap();

      let resolved_path_str = format!("\"{}\"", resolved_path);
      // TODO(sagar): fix source map?
      node.src = Some(Box::new(Str {
        raw: Some(resolved_path_str.clone().into()),
        span: src.span,
        value: resolved_path_str.into(),
      }));
    }
  }
}
