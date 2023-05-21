use deno_ast::swc::atoms::JsWord;
use swc_common::BytePos;
use swc_common::Span;
use swc_ecma_ast::BindingIdent;
use swc_ecma_ast::ExportDefaultExpr;
use swc_ecma_ast::Expr;
use swc_ecma_ast::Ident;
use swc_ecma_ast::MemberExpr;
use swc_ecma_ast::ModuleDecl;
use swc_ecma_ast::ModuleItem;
use swc_ecma_ast::ObjectLit;
use swc_ecma_ast::Stmt;
use swc_ecma_ast::VarDecl;
use swc_ecma_ast::VarDeclKind;
use swc_ecma_ast::VarDeclarator;
use swc_ecma_visit::VisitWith;
use swc_ecma_visit::{as_folder, Fold, Visit, VisitMut};

pub(crate) struct CommonJsToEsm;

/// Super hacky way to convert CommonJs to ESM
/// Only works in very few cases
pub fn to_esm() -> impl Fold + VisitMut {
  as_folder(CommonJsToEsm)
}

impl VisitMut for CommonJsToEsm {
  fn visit_mut_module(&mut self, node: &mut swc_ecma_ast::Module) {
    let mut folder = CommonJsChecker {
      module: JsWord::from("module"),
      exports: JsWord::from("exports"),
      is_commonjs: false,
    };
    node.visit_children_with(&mut folder);

    if !folder.is_commonjs {
      return;
    }
    let var_decl = VarDeclarator {
      name: swc_ecma_ast::Pat::Ident(BindingIdent {
        id: Ident {
          span: Span {
            lo: BytePos(0),
            hi: BytePos(1),
            ctxt: node.span.ctxt,
          },
          sym: JsWord::from("module"),
          optional: false,
        },
        type_ann: None,
      }),
      init: Some(
        Expr::Object(ObjectLit {
          span: Span {
            lo: BytePos(0),
            hi: BytePos(1),
            ctxt: node.span.ctxt,
          },
          props: vec![],
        })
        .into(),
      ),
      span: Span {
        lo: BytePos(0),
        hi: BytePos(1),
        ctxt: node.span.ctxt,
      },
      definite: false,
    };

    let module_decl = ModuleItem::Stmt(Stmt::Decl(
      VarDecl {
        kind: VarDeclKind::Let,
        span: Span {
          lo: BytePos(0),
          hi: BytePos(1),
          ctxt: node.span.ctxt,
        },
        declare: false,
        decls: vec![var_decl],
      }
      .into(),
    ));

    let export_decl = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
      ExportDefaultExpr {
        span: Span {
          lo: BytePos(0),
          hi: BytePos(1),
          ctxt: node.span.ctxt,
        },
        expr: Ident {
          span: Span {
            lo: BytePos(0),
            hi: BytePos(1),
            ctxt: node.span.ctxt,
          },
          sym: JsWord::from("module.exports"),
          optional: false,
        }
        .into(),
      },
    ));

    node.body.insert(0, module_decl);
    node.body.push(export_decl);
  }
}

pub(crate) struct CommonJsChecker {
  module: JsWord,
  exports: JsWord,
  is_commonjs: bool,
}

impl Visit for CommonJsChecker {
  fn visit_member_expr(&mut self, node: &MemberExpr) {
    match (node.obj.as_ident(), node.prop.as_ident()) {
      (
        Some(Ident {
          span: _,
          sym: obj_sym,
          optional: _,
        }),
        Some(Ident {
          span: _,
          sym: prop_sym,
          optional: _,
        }),
      ) => {
        if *obj_sym == self.module && *prop_sym == self.exports {
          self.is_commonjs = true;
        }
      }
      _ => {}
    }
  }
}
