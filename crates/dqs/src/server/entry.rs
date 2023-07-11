use anyhow::Result;
use deno_core::{ModuleCode, ModuleSpecifier};
use url::Url;

#[derive(Debug, Clone, Default)]
pub enum ServerEntry {
  /// This is a server entry for widgets, etc
  DqsServer,
  /// This is a server entry for an app template's router
  #[allow(dead_code)]
  AppServer,

  #[default]
  Noop,
}

impl ServerEntry {
  pub fn get_main_module(&self) -> Result<(ModuleSpecifier, ModuleCode)> {
    match self {
      ServerEntry::DqsServer => Ok((
        Url::parse("builtin://main")?,
        include_str!("./entry_query_server.js").to_owned().into(),
      )),
      Self::AppServer => Ok((
        Url::parse("builtin://main")?,
        include_str!("../../../../js/arena-runtime/dist/app-server.js")
          .to_owned()
          .into(),
      )),
      Self::Noop => Ok((Url::parse("builtin://main")?, format!("").into())),
    }
  }
}
