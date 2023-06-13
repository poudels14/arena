use self::parser::parse;
use crate::ast::emitter::ToString;
use crate::ast::types::*;
use anyhow::Result;
use swc_ecma_ast::{Expr, Module, Prop, PropOrSpread};

pub(crate) mod args_sanitizer;
pub mod parser;
pub(crate) mod props_sanitizer;

#[derive(Debug)]
pub struct DataQuery {
  pub module: Module,
  pub unresolved_exprs: Vec<Expr>,
}

impl DataQuery {
  pub fn from(code: &str) -> Result<DataQuery> {
    parse(code)
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

  #[test]
  fn test_args_generator() {
    let query = parser::parse(
      r#"
      export default function() {
        return input.value
      }
      "#,
    )
    .unwrap();
    assert_eq!(
      "return {    input: {        value: input.value    }};",
      query.get_props_generator().unwrap()
    );
  }
}
