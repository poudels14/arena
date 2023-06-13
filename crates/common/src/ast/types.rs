use swc_atoms::JsWord;
use swc_common::DUMMY_SP;
use swc_ecma_ast::{
  AssignPatProp, Expr, Ident, KeyValueProp, MemberExpr, MemberProp, ObjectLit,
  ObjectPat, ObjectPatProp, Param, Pat, Prop, PropName, PropOrSpread,
  ReturnStmt,
};

pub fn ident(sym: &str) -> Ident {
  Ident {
    span: DUMMY_SP,
    sym: JsWord::from(sym),
    optional: false,
  }
}

pub fn member_expr(obj: Expr, prop: MemberProp) -> MemberExpr {
  MemberExpr {
    span: DUMMY_SP,
    obj: obj.into(),
    prop,
  }
}

pub fn param(pat: Pat) -> Param {
  Param {
    span: DUMMY_SP,
    decorators: vec![],
    pat,
  }
}

pub fn object_pat(props: Vec<ObjectPatProp>) -> ObjectPat {
  ObjectPat {
    span: DUMMY_SP,
    props,
    optional: false,
    type_ann: None,
  }
}

pub fn assign_object_pat_prop(key: Ident) -> ObjectPatProp {
  ObjectPatProp::Assign(assign_pat_prop(key))
}

pub fn assign_pat_prop(key: Ident) -> AssignPatProp {
  AssignPatProp {
    span: DUMMY_SP,
    key,
    value: None,
  }
}

pub fn object_lit(props: Vec<PropOrSpread>) -> ObjectLit {
  ObjectLit {
    span: DUMMY_SP,
    props,
  }
}

pub fn key_value_prop(key: Ident, value: Expr) -> Prop {
  Prop::KeyValue(KeyValueProp {
    key: PropName::Ident(key),
    value: value.into(),
  })
}

pub fn return_stmt(arg: Expr) -> ReturnStmt {
  ReturnStmt {
    span: DUMMY_SP,
    arg: Some(arg.into()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::ast::emitter::ToString;

  #[test]
  fn test_membr_expr() {
    let membr = member_expr(
      Expr::Ident(ident("props")),
      MemberProp::Ident(ident("data")),
    );
    assert_eq!("props.data;", membr.to_string().unwrap());
  }

  #[test]
  fn test_object_return() {
    let membr =
      return_stmt(Expr::Object(object_lit(vec![PropOrSpread::Prop(
        key_value_prop(
          ident("input"),
          Expr::Object(object_lit(vec![PropOrSpread::Prop(
            key_value_prop(
              ident("text"),
              Expr::Member(member_expr(
                Expr::Ident(ident("input")),
                MemberProp::Ident(ident("text")),
              )),
            )
            .into(),
          )])),
        )
        .into(),
      )])));
    assert_eq!(
      "return {    input: {        text: input.text    }};",
      membr.to_string().unwrap()
    );
  }
}
