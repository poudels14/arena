use deno_ast::swc::atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_ecma_ast::ArrowExpr;
use swc_ecma_ast::AssignExpr;
use swc_ecma_ast::BinExpr;
use swc_ecma_ast::BindingIdent;
use swc_ecma_ast::BlockStmt;
use swc_ecma_ast::BlockStmtOrExpr;
use swc_ecma_ast::CallExpr;
use swc_ecma_ast::Callee;
use swc_ecma_ast::ExportDefaultExpr;
use swc_ecma_ast::Expr;
use swc_ecma_ast::ExprOrSpread;
use swc_ecma_ast::Ident;
use swc_ecma_ast::KeyValueProp;
use swc_ecma_ast::MemberExpr;
use swc_ecma_ast::MemberProp;
use swc_ecma_ast::ModuleDecl;
use swc_ecma_ast::ModuleItem;
use swc_ecma_ast::ObjectLit;
use swc_ecma_ast::Pat;
use swc_ecma_ast::Prop;
use swc_ecma_ast::PropName;
use swc_ecma_ast::PropOrSpread;
use swc_ecma_ast::SeqExpr;
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

    let export_decl = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
      ExportDefaultExpr {
        span: DUMMY_SP,
        expr: Ident {
          span: DUMMY_SP,
          sym: JsWord::from("require_module()"),
          optional: false,
        }
        .into(),
      },
    ));

    let body = node
      .body
      .iter_mut()
      .map(move |m| match m {
        ModuleItem::ModuleDecl(_) => {
          panic!("ModuleDecl not supported in commonjs module")
        }
        ModuleItem::Stmt(stmt) => stmt.to_owned(),
      })
      .collect::<Vec<Stmt>>();

    // essentially, transform commonjs module to following
    // `
    //    let __commonJS = ...;
    //    let require_module = __commonJS(...);
    //    export default require_module();
    // `
    // this is how bun seems to do it, at least on top level
    // not sure if the module code itself is modified
    node.body = vec![
      ModuleItem::Stmt(Stmt::Decl(
        VarDecl {
          kind: VarDeclKind::Let,
          span: DUMMY_SP,
          declare: false,
          decls: vec![get_common_js_function_ast()],
        }
        .into(),
      )),
      ModuleItem::Stmt(Stmt::Decl(
        VarDecl {
          kind: VarDeclKind::Let,
          span: DUMMY_SP,
          declare: false,
          decls: vec![get_common_js_initializer(
            BlockStmtOrExpr::BlockStmt(BlockStmt {
              span: DUMMY_SP,
              stmts: body,
            })
            .into(),
          )],
        }
        .into(),
      )),
      export_decl,
    ]
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

fn get_common_js_initializer(body: Box<BlockStmtOrExpr>) -> VarDeclarator {
  VarDeclarator {
    span: DUMMY_SP,
    definite: false,
    name: Pat::Ident(BindingIdent {
      id: Ident {
        span: DUMMY_SP,
        sym: JsWord::from("require_module"),
        optional: false,
      },
      type_ann: None,
    }),
    init: Some(
      Expr::Call(CallExpr {
        span: DUMMY_SP,
        type_args: None,
        callee: Callee::Expr(
          Expr::Ident(Ident {
            span: DUMMY_SP,
            sym: JsWord::from("__commonJS"),
            optional: false,
          })
          .into(),
        ),
        args: vec![ExprOrSpread {
          spread: None,
          expr: Expr::Arrow(ArrowExpr {
            span: DUMMY_SP,
            params: vec![
              Pat::Ident(BindingIdent {
                type_ann: None,
                id: Ident {
                  span: DUMMY_SP,
                  sym: JsWord::from("exports"),
                  optional: false,
                },
              }),
              Pat::Ident(BindingIdent {
                type_ann: None,
                id: Ident {
                  span: DUMMY_SP,
                  sym: JsWord::from("module"),
                  optional: false,
                },
              }),
            ],
            body,
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
          })
          .into(),
        }],
      })
      .into(),
    ),
  }
}

/// This returns AST for the following function
/// `var __commonJS = (cb, mod) => () =>
///     (mod || cb((mod = { exports: {} }).exports, mod), mod.exports);`
fn get_common_js_function_ast() -> VarDeclarator {
  VarDeclarator {
    span: DUMMY_SP,
    definite: false,
    name: Pat::Ident(BindingIdent {
      id: Ident {
        span: DUMMY_SP,
        sym: JsWord::from("__commonJS"),
        optional: false,
      },
      type_ann: None,
    }),
    init: Some(
      Expr::Arrow(ArrowExpr {
        span: DUMMY_SP,
        params: vec![
          Pat::Ident(BindingIdent {
            type_ann: None,
            id: Ident {
              span: DUMMY_SP,
              sym: JsWord::from("cb"),
              optional: false,
            },
          }),
          Pat::Ident(BindingIdent {
            type_ann: None,
            id: Ident {
              span: DUMMY_SP,
              sym: JsWord::from("mod"),
              optional: false,
            },
          }),
        ],
        body: BlockStmtOrExpr::Expr(
          Expr::Arrow(ArrowExpr {
            span: DUMMY_SP,
            params: vec![],
            body: BlockStmtOrExpr::Expr(
              Expr::Seq(SeqExpr {
                span: DUMMY_SP,
                exprs: vec![
                  Expr::Bin(BinExpr {
                    span: DUMMY_SP,
                    op: swc_ecma_ast::BinaryOp::LogicalOr,
                    left: Expr::Ident(Ident {
                      span: DUMMY_SP,
                      sym: JsWord::from("mod"),
                      optional: false,
                    })
                    .into(),
                    right: Expr::Call(CallExpr {
                      span: DUMMY_SP,
                      args: vec![
                        ExprOrSpread {
                          spread: None,
                          expr: Expr::Member(MemberExpr {
                            span: DUMMY_SP,
                            obj: Expr::Assign(AssignExpr {
                              span: DUMMY_SP,
                              op: swc_ecma_ast::AssignOp::Assign,
                              left: swc_ecma_ast::PatOrExpr::Pat(
                                Pat::Ident(BindingIdent {
                                  id: Ident {
                                    span: DUMMY_SP,
                                    sym: JsWord::from("mod"),
                                    optional: false,
                                  },
                                  type_ann: None,
                                })
                                .into(),
                              ),
                              right: Expr::Object(ObjectLit {
                                span: DUMMY_SP,
                                props: vec![PropOrSpread::Prop(
                                  Prop::KeyValue(KeyValueProp {
                                    key: PropName::Ident(Ident {
                                      span: DUMMY_SP,
                                      sym: JsWord::from("exports"),
                                      optional: false,
                                    }),
                                    value: Expr::Object(ObjectLit {
                                      span: DUMMY_SP,
                                      props: vec![],
                                    })
                                    .into(),
                                  })
                                  .into(),
                                )
                                .into()],
                              })
                              .into(),
                            })
                            .into(),
                            prop: MemberProp::Ident(Ident {
                              span: DUMMY_SP,
                              sym: JsWord::from("exports"),
                              optional: false,
                            }),
                          })
                          .into(),
                        },
                        ExprOrSpread {
                          spread: None,
                          expr: Expr::Ident(Ident {
                            span: DUMMY_SP,
                            sym: JsWord::from("mod"),
                            optional: false,
                          })
                          .into(),
                        },
                      ],
                      type_args: None,
                      callee: Callee::Expr(
                        Expr::Ident(Ident {
                          span: DUMMY_SP,
                          sym: JsWord::from("cb"),
                          optional: false,
                        })
                        .into(),
                      ),
                    })
                    .into(),
                  })
                  .into(),
                  Expr::Member(MemberExpr {
                    span: DUMMY_SP,
                    obj: Expr::Ident(Ident {
                      span: DUMMY_SP,
                      sym: JsWord::from("mod"),
                      optional: false,
                    })
                    .into(),
                    prop: MemberProp::Ident(Ident {
                      span: DUMMY_SP,
                      sym: JsWord::from("exports"),
                      optional: false,
                    }),
                  })
                  .into(),
                ],
              })
              .into(),
            )
            .into(),
            is_async: false,
            is_generator: false,
            type_params: None,
            return_type: None,
          })
          .into(),
        )
        .into(),
        is_async: false,
        is_generator: false,
        type_params: None,
        return_type: None,
      })
      .into(),
    ),
  }
}
