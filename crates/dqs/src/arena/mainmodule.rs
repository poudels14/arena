use anyhow::Result;
use deno_core::{ModuleCode, ModuleSpecifier};
use url::Url;

use super::app::App;
use super::workflow::PluginWorkflow;

#[derive(Debug, Clone)]
pub enum MainModule {
  WidgetQuery,
  App {
    app: App,
  },
  PluginWorkflowRun {
    workflow: PluginWorkflow,
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

  pub fn get_entry_module(
    &self,
  ) -> Result<(ModuleSpecifier, Option<ModuleCode>)> {
    match self {
      Self::WidgetQuery => {
        Ok((Url::parse("builtin:///@arena/dqs/widget-server")?, None))
      }
      Self::App { app: _ } => Ok((
        Url::parse("builtin:///main")?,
        Some(
          include_str!("../../../../js/runtime/dist/dqs/app-server.js")
            .to_owned()
            .into(),
        ),
      )),
      // Self::PluginWorkflowRun { workflow: _ } => Ok((
      //   Url::parse("builtin:///main")?,
      //   Some(
      //     include_str!("../../../../js/runtime/dist/dqs/plugin-workflow.js")
      //       .to_owned()
      //       .into(),
      //   ),
      // )),
      Self::Inline { code } => {
        Ok((Url::parse("builtin:///main")?, Some(code.clone().into())))
      }
      _ => unimplemented!(),
    }
  }
}
