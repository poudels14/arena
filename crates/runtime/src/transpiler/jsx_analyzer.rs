use swc_ecma_visit::Visit;

pub struct JsxAnalyzer {
  pub is_react: bool,
}

impl JsxAnalyzer {
  pub fn new() -> Self {
    Self { is_react: false }
  }
}

impl Visit for JsxAnalyzer {
  fn visit_import_decl(&mut self, node: &swc_ecma_ast::ImportDecl) {
    // Note(sagar): if a default specifier is imported from 'react', consider
    // it a React module
    if node.src.value.eq("react") {
      self.is_react = true;
    }
  }
}
