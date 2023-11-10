use anyhow::Result;
use std::rc::Rc;
use swc_common::{SourceMap, DUMMY_SP};
use swc_ecma_ast::{
  EsVersion, Expr, ExprStmt, Module, ModuleItem, ObjectLit, ReturnStmt,
};
use swc_ecma_ast::{MemberExpr, Stmt};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};

pub trait ToString {
  fn to_string(self) -> Result<String>;
}

impl ToString for MemberExpr {
  fn to_string(self) -> Result<String> {
    emit_expr(Expr::Member(self))
  }
}

impl ToString for ObjectLit {
  fn to_string(self) -> Result<String> {
    emit_expr(Expr::Object(self))
  }
}

impl ToString for ReturnStmt {
  fn to_string(self) -> Result<String> {
    emit_stmt(Stmt::Return(self))
  }
}

impl ToString for Module {
  fn to_string(self) -> Result<String> {
    let mut buf = vec![];
    let mut emitter = create_emitter(&mut buf);
    emitter.emit_module(&self)?;
    Ok(String::from_utf8(buf)?)
  }
}

impl ToString for Expr {
  fn to_string(self) -> Result<String> {
    emit_expr(self)
  }
}

pub fn emit_expr(expr: Expr) -> Result<String> {
  emit_stmt(Stmt::Expr(ExprStmt {
    span: DUMMY_SP,
    expr: expr.into(),
  }))
}

pub fn emit_stmt(stmt: Stmt) -> Result<String> {
  let mut buf = vec![];
  let mut emitter = create_emitter(&mut buf);
  emitter.emit_module_item(&ModuleItem::Stmt(stmt))?;
  Ok(String::from_utf8(buf).unwrap())
}

fn create_emitter<'a>(
  buf: &'a mut Vec<u8>,
) -> Emitter<'a, Box<JsWriter<'a, &'a mut Vec<u8>>>, SourceMap> {
  let cm = Rc::new(SourceMap::default());
  let wr = Box::new(JsWriter::new(cm.clone(), "", buf, None));
  let cfg = swc_ecma_codegen::Config::default()
    .with_minify(false)
    .with_ascii_only(false)
    .with_omit_last_semi(true)
    .with_target(EsVersion::Es2021);

  swc_ecma_codegen::Emitter {
    cfg,
    comments: None,
    cm,
    wr,
  }
}
