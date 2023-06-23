use std::sync::OnceLock;

use self::parser::parse;
use crate::ast::emitter::ToString;
use crate::ast::types::*;
use anyhow::Result;
use indexmap::{indexset, IndexSet};
use swc_atoms::JsWord;
use swc_ecma_ast::{Expr, Module, Prop, PropOrSpread};

pub(crate) mod args_sanitizer;
pub mod parser;
pub(crate) mod props_sanitizer;

#[derive(Debug)]
pub struct DataQuery {
  pub module: Module,
  pub unresolved_exprs: Vec<Expr>,
}

static INIT: OnceLock<IndexSet<JsWord>> = OnceLock::new();

fn get_allowed_global_idents() -> IndexSet<JsWord> {
  indexset! {
    JsWord::from("JSON"),
    JsWord::from("Boolean"),
    JsWord::from("parseInt"),
    JsWord::from("console"),
    JsWord::from("fetch"),
    JsWord::from("Date")
  }
}

impl DataQuery {
  pub fn from(code: &str) -> Result<DataQuery> {
    parse(code, &INIT.get_or_init(get_allowed_global_idents))
  }

  pub fn get_props_generator(&self) -> Result<String> {
    let props = self
      .unresolved_exprs
      .iter()
      .map(|node| match node {
        Expr::Member(node) => Ok(PropOrSpread::Prop(
          membr_to_key_value(
            node.obj.as_ref(),
            Expr::Object(object_lit(vec![PropOrSpread::Prop(
              key_value_prop(
                node.prop.clone().expect_ident(),
                Expr::Member(node.clone()),
              )
              .into(),
            )])),
          )
          .into(),
        )),
        _ => {
          unimplemented!()
        }
      })
      .collect::<Result<Vec<PropOrSpread>>>()?;
    return_stmt(Expr::Object(object_lit(props))).to_string()
  }

  pub fn get_server_module(&self) -> Result<String> {
    self.module.clone().to_string()
  }
}

// This converts `input.text` to `{ input: { text: input.text }}`
fn membr_to_key_value(node: &Expr, value: Expr) -> Prop {
  match node {
    Expr::Member(m) => membr_to_key_value(m.obj.as_ref(), value),
    Expr::Ident(ident) => key_value_prop(ident.clone(), value),
    _ => unimplemented!(),
  }
}

#[cfg(test)]
mod tests {
  use crate::query::parser;
  use indexmap::indexset;

  #[test]
  fn test_args_generator() {
    let query = parser::parse(
      r#"
      export default function() {
        return input.value
      }
      "#,
      &indexset! {},
    )
    .unwrap();
    assert_eq!(
      "return {    input: {        value: input.value    }};",
      query.get_props_generator().unwrap()
    );
  }
}
