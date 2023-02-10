use anyhow::{anyhow, bail, Result};
use swc_common::sync::Lrc;
use swc_common::Globals;
use swc_common::Mark;
use swc_common::GLOBALS;
use swc_common::{FileName, SourceMap};
use swc_ecma_ast::Module;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::TsConfig;
use swc_ecma_parser::{Parser, StringInput, Syntax};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::FoldWith;

pub struct Analyzer {
  top_level_mark: Option<Mark>,
}

pub struct Options {
  pub strip_typescript: bool,
}

pub struct Report {
  pub module: Module,
  pub source_map: Lrc<SourceMap>,
}

impl Analyzer {
  pub fn new() -> Self {
    Self {
      top_level_mark: None,
    }
  }

  pub fn analyze(
    &mut self,
    filename: &str,
    code: &str,
    options: &Options,
  ) -> Result<Box<Report>> {
    let cm: Lrc<SourceMap> = Default::default();

    let fm = cm.new_source_file(
      FileName::Custom(filename.to_string()),
      code.to_string(),
    );
    let lexer = Lexer::new(
      Syntax::Typescript(TsConfig {
        tsx: true,
        ..Default::default()
      }),
      Default::default(),
      StringInput::from(&*fm),
      None,
    );

    let mut parser = Parser::new_from(lexer);

    // Note(sagar): return first error
    if let Some(e) = parser.take_errors().pop() {
      bail!("{:?}", e);
    }

    let parsed_module =
      parser.parse_module().map_err(|e| anyhow!("{:?}", e))?;

    GLOBALS.set(&Globals::new(), || {
      let top_level_mark = Mark::fresh(Mark::root());
      self.top_level_mark = Some(top_level_mark);
      let module = match options.strip_typescript {
        true => parsed_module.fold_with(&mut strip(top_level_mark)),
        false => parsed_module,
      };

      Ok(Box::new(Report {
        source_map: cm,
        module,
      }))
    })
  }
}
