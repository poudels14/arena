use super::super::super::resolver::resolve;
use super::super::Transpiler;
use anyhow::{anyhow, Result};
use deno_core::normalize_path;
use std::borrow::Borrow;
use std::path::Path;
use std::rc::Rc;
use swc_ecma_ast::{CallExpr, Callee, Expr, Lit, Str};
use swc_ecma_ast::{ExportAll, ImportDecl};
use swc_ecma_visit::VisitMutWith;
use swc_ecma_visit::{as_folder, Fold, VisitMut};

pub(crate) struct Resolver {
  transpiler: Rc<Transpiler>,
  referrer: String,
}

pub(crate) fn init(
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
    let src = &node.src;
    if let Ok(resolved_path_str) =
      resolve_path(&self.transpiler, &self.referrer, src.value.as_ref())
    {
      // TODO(sagar): fix source map?
      node.src = Box::new(Str {
        raw: Some(resolved_path_str.clone().into()),
        span: src.span,
        value: resolved_path_str.into(),
      });
    }
  }

  /// This resolves file path inside `import(...)` statements
  fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
    match node.callee {
      Callee::Import(_) => {
        let src = &node.args[0].expr;
        if let Some(lit) = src.as_lit() {
          match lit {
            Lit::Str(Str {
              span,
              value,
              raw: _,
            }) => {
              let resolved_path_str =
                resolve_path(&self.transpiler, &self.referrer, value.as_ref());
              if let Ok(resolved_path_str) = resolved_path_str {
                node.args[0].expr = Box::new(Expr::Lit(Lit::Str(Str {
                  span: span.to_owned(),
                  value: resolved_path_str.clone().into(),
                  raw: Some(resolved_path_str.into()),
                })))
              }
            }
            _ => {}
          }
        }
      }
      _ => {
        node.visit_mut_children_with(self);
      }
    }
  }

  fn visit_mut_export_all(&mut self, node: &mut ExportAll) {
    let src = &node.src;
    let resolved_path_str =
      resolve_path(&self.transpiler, &self.referrer, src.value.as_ref());
    if let Ok(resolved_path_str) = resolved_path_str {
      // TODO(sagar): fix source map?
      node.src = Box::new(Str {
        raw: Some(resolved_path_str.clone().into()),
        span: src.span,
        value: resolved_path_str.into(),
      });
    }
  }

  fn visit_mut_named_export(&mut self, node: &mut swc_ecma_ast::NamedExport) {
    if let Some(src) = &node.src {
      let resolved_path_str =
        resolve_path(&self.transpiler, &self.referrer, src.value.as_ref());

      if let Ok(resolved_path_str) = resolved_path_str {
        // TODO(sagar): fix source map?
        node.src = Some(Box::new(Str {
          raw: Some(resolved_path_str.clone().into()),
          span: src.span,
          value: resolved_path_str.into(),
        }));
      }
    }
  }
}

fn resolve_path(
  transpiler: &Transpiler,
  referrer: &str,
  specifier: &str,
) -> Result<String> {
  let resolved_path = resolve(
    transpiler.resolver.borrow(),
    &transpiler.root,
    referrer,
    specifier,
  )?;

  // Note(sagar): prefix resolved path with "/" so that all resolved paths
  // are absolute path from project root
  normalize_path(
    Path::new("/").join(resolved_path.unwrap_or(specifier.to_string())),
  )
  .to_str()
  .map(|s| format!("\"{s}\""))
  .ok_or(anyhow!("error resolving path"))
}
