use super::props_sanitizer::PropsSanitizer;
use super::DataQuery;
use crate::query::args_sanitizer::ArgsSanitizer;
use anyhow::{anyhow, bail, Result};
use deno_ast::swc::parser::lexer::Lexer;
use indexmap::IndexSet;
use swc_atoms::JsWord;
use swc_common::sync::Lrc;
use swc_common::{chain, FileName, Mark, SourceMap};
use swc_ecma_parser::{Parser as SwcParser, StringInput, Syntax};
use swc_ecma_transforms_base::resolver;
use swc_ecma_visit::{FoldWith, VisitMutWith};

pub fn parse(
  code: &str,
  known_globals: &IndexSet<JsWord>,
) -> Result<DataQuery> {
  let cm: Lrc<SourceMap> = Default::default();

  let fm = cm
    .new_source_file(FileName::Custom("code.js".to_string()), code.to_string());
  let lexer = Lexer::new(
    Syntax::Typescript(Default::default()),
    Default::default(),
    StringInput::from(&*fm),
    None,
  );

  let mut swc_parser = SwcParser::new_from(lexer);

  // Note(sagar): return first error
  if let Some(e) = swc_parser.take_errors().pop() {
    bail!("{:?}", e);
  }

  swc_common::GLOBALS.set(&swc_common::Globals::new(), || {
    let unresolved_mark = Mark::new();
    let top_level_mark = Mark::fresh(Mark::root());

    let module = swc_parser.parse_module().map_err(|e| anyhow!("{:?}", e))?;
    let mut props_prefixer = PropsSanitizer {
      known_globals: &known_globals,
      unresolved_mark,
      unresolved_exprs: vec![],
    };

    let mut program =
      module.fold_with(&mut resolver(unresolved_mark, top_level_mark, true));
    // println!("program = {:#?}", program);
    program.visit_mut_with(&mut chain!(&mut props_prefixer, ArgsSanitizer {}));

    Ok(DataQuery {
      module: program,
      unresolved_exprs: props_prefixer.unresolved_exprs,
    })
  })
}

#[cfg(test)]
mod tests {
  use indexmap::indexset;

  use crate::ast::emitter::ToString;
  use crate::query::parser;

  /// Test analyzing unresolved objects
  #[test]
  fn test_props_prefixing() {
    let query = parser::parse(
      r#"
      export default function({ env }) {
        return input.text.value;
      }
      "#,
      &indexset! {},
    )
    .unwrap();
    assert_eq!(1, query.unresolved_exprs.len());
    assert_eq!(
      "input.text.value;",
      query.unresolved_exprs[0].clone().to_string().unwrap()
    );
  }

  #[test]
  fn test_default_fn_arg_sanitization() {
    let query = parser::parse(
      "export default function() { return input.value }",
      &indexset! {},
    )
    .unwrap();

    assert_eq!(
      "export default function({ env, props }) {    return props.input.value;}",
      query.get_server_module().unwrap()
    );
  }
}
