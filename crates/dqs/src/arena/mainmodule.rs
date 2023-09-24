use std::rc::Rc;

use anyhow::Result;
use common::deno::extensions::BuiltinModule;
use deno_core::{ModuleCode, ModuleSpecifier};
use url::Url;

use super::app::App;
use super::template::Template;
use crate::arena;

#[derive(Debug, Clone)]
pub enum MainModule {
  WidgetQuery,
  App {
    app: App,
  },
  Workflow {
    id: String,
    name: String,
    plugin: Template,
  },
  #[allow(dead_code)]
  /// This is used for testing only
  Inline {
    code: String,
  },
}

impl MainModule {
  pub fn as_app<'a>(&'a self) -> Option<&'a App> {
    match self {
      Self::App { app } => Some(app),
      _ => None,
    }
  }

  pub fn get_builtin_module_extensions(&self) -> Vec<BuiltinModule> {
    match self {
      Self::App { app: _ } => {
        vec![BuiltinModule::Custom(Rc::new(arena::extension))]
      }
      _ => vec![],
    }
  }

  pub fn get_entry_module(&self) -> Result<(ModuleSpecifier, ModuleCode)> {
    match self {
      Self::WidgetQuery => Ok((
        Url::parse("builtin:///main")?,
        include_str!("../../../../js/arena-runtime/dist/dqs/widget-server.js")
          .to_owned()
          .into(),
      )),
      Self::App { app: _ } => Ok((
        Url::parse("builtin:///main")?,
        include_str!("../../../../js/arena-runtime/dist/dqs/app-server.js")
          .to_owned()
          .into(),
      )),
      Self::Workflow {
        id: _,
        name: _,
        plugin: _,
      } => Ok((
        Url::parse("builtin:///main")?,
        include_str!(
          "../../../../js/arena-runtime/dist/dqs/plugin-workflow.js"
        )
        .to_owned()
        .into(),
      )),
      Self::Inline { code } => {
        Ok((Url::parse("builtin:///main")?, code.clone().into()))
      }
    }
  }
}
