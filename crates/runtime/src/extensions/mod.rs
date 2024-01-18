mod r#macro;

pub mod babel;
pub mod bundler;
pub mod env;
pub mod fs;
pub mod node;
pub mod postgres;
pub mod resolver;
pub mod rollup;
pub mod server;
pub mod sqlite;
pub mod transpiler;

use anyhow::Result;
use deno_core::{Extension, JsRuntime, ModuleCode};
use derivative::Derivative;
use derive_new::new;
use indexmap::IndexSet;
use std::rc::Rc;
use tracing::debug;
use url::Url;

use self::server::HttpServerConfig;
use crate::config::node::ResolverConfig;

#[derive(Default, new)]
pub struct BuiltinExtension {
  pub extension: Option<Extension>,
  /// tuples of module's (specifier, source_code)
  pub modules: Vec<(&'static str, SourceCode)>,
}

// NOTE: don't use this directly. Instead use one of the macros
#[derive(Debug, Clone)]
pub enum SourceCode {
  /// use this if the source code should be preserved and loaded
  /// when when the extension is initialied
  Preserved(&'static str),
  /// use this if the module is already snapshotted and the source
  /// code is no longer needed
  #[cfg(not(feature = "include-in-binary"))]
  NotPreserved,
}

impl SourceCode {
  pub fn code(&self) -> &'static str {
    match self {
      Self::Preserved(s) => s,
      #[cfg(not(feature = "include-in-binary"))]
      Self::NotPreserved => panic!("Source code not included"),
    }
  }
}

pub trait BuiltinExtensionProvider {
  fn get_extension(&self) -> BuiltinExtension;
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
#[allow(unused)]
pub enum BuiltinModule {
  Fs,
  Env,
  Node(Option<Vec<&'static str>>),
  Resolver(ResolverConfig),
  Transpiler,
  Babel,
  Postgres,
  Sqlite,
  HttpServer(HttpServerConfig),
  /// args: (specifier, code)
  CustomRuntimeModule(&'static str, SourceCode),
  UsingProvider(
    #[derivative(Debug = "ignore")] Rc<dyn BuiltinExtensionProvider>,
  ),
  Custom(#[derivative(Debug = "ignore")] Rc<dyn Fn() -> BuiltinExtension>),
}

impl BuiltinExtensionProvider for BuiltinModule {
  fn get_extension(&self) -> BuiltinExtension {
    match self {
      Self::Fs => self::fs::extension(),
      Self::Env => self::env::extension(),
      Self::Node(filter) => self::node::extension(filter.to_owned()),
      Self::Resolver(config) => self::resolver::extension(config),
      Self::Transpiler => self::transpiler::extension(),
      Self::Babel => self::babel::extension(),
      Self::Postgres => self::postgres::extension(),
      Self::Sqlite => self::sqlite::extension(),
      Self::HttpServer(config) => self::server::extension(config.clone()),
      Self::CustomRuntimeModule(specifier, code) => BuiltinExtension {
        modules: vec![(specifier, code.clone())],
        ..Default::default()
      },
      Self::UsingProvider(p) => p.get_extension(),
      Self::Custom(ext) => ext(),
    }
  }
}

#[derive(Default)]
pub struct BuiltinExtensions {}

#[allow(unused)]
impl BuiltinExtensions {
  pub fn all() -> Vec<BuiltinExtension> {
    vec![
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
      BuiltinModule::Resolver(Default::default()),
      BuiltinModule::Transpiler,
      BuiltinModule::Babel,
      BuiltinModule::HttpServer(HttpServerConfig::Tcp {
        address: "0.0.0.0".to_owned(),
        port: 0,
        serve_dir: None,
      }),
    ]
    .into_iter()
    .map(|m| m.get_extension())
    .collect()
  }

  pub fn load_extensions<'a>(
    extensions: &Vec<BuiltinExtension>,
    runtime: &mut JsRuntime,
  ) -> Result<()> {
    futures::executor::block_on(async {
      for extension in extensions.iter() {
        for module in &extension.modules {
          let (specifier, code) = module;
          debug!(
            "Loading built-in module into the runtime: {:?}, code len = {}",
            specifier,
            code.code().len()
          );
          let mod_id = runtime
            .load_side_module(
              &Url::parse(&format!("builtin://{}", specifier))?,
              ModuleCode::from_static(code.code()).into(),
            )
            .await?;
          let receiver = runtime.mod_evaluate(mod_id);
          receiver.await?;
        }
      }
      Ok(())
    })
  }

  pub fn get_deno_extensions(
    extensions: &mut Vec<BuiltinExtension>,
  ) -> Vec<Extension> {
    extensions
      .iter_mut()
      .map(|e| e.extension.take())
      .filter(|e| e.is_some())
      .map(|e| e.unwrap())
      .collect()
  }

  pub fn get_specifiers(
    extensions: &Vec<BuiltinExtension>,
  ) -> IndexSet<String> {
    extensions
      .iter()
      .map(|e| {
        e.modules
          .iter()
          .map(|m| m.0.to_string())
          .collect::<Vec<String>>()
      })
      .flatten()
      .collect::<IndexSet<String>>()
  }
}
