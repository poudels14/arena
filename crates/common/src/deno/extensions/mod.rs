pub mod babel;
pub mod env;
mod extension;
pub mod fs;
pub mod node;
pub mod postgres;
pub mod resolver;
pub mod rollup;
pub mod server;
pub mod transpiler;
pub mod wasi;

use self::extension::BuiltinExtension;
use anyhow::Result;
use deno_core::{Extension, ExtensionFileSourceCode, JsRuntime};
use indexmap::IndexSet;
use std::path::PathBuf;
use tracing::debug;
use url::Url;

pub enum BuiltinModule<'a> {
  Fs,
  Env,
  Node,
  Resolver(PathBuf),
  Transpiler,
  Babel,
  Rollup,
  Postgres,
  /// (address, port)
  HttpServer(&'a str, u16),
  /// args: (specifier, code)
  CustomRuntimeModule(&'static str, &'static str),
  Custom(fn() -> BuiltinExtension),
}

impl<'a> BuiltinModule<'a> {
  pub(crate) fn extension(&self) -> BuiltinExtension {
    match self {
      Self::Fs => self::fs::extension(),
      Self::Env => self::env::extension(),
      Self::Node => self::node::extension(),
      Self::Resolver(root) => self::resolver::extension(root.clone()),
      Self::Transpiler => self::transpiler::extension(),
      Self::Babel => self::babel::extension(),
      Self::Rollup => self::rollup::extension(),
      Self::Postgres => self::postgres::extension(),
      Self::HttpServer(address, port) => {
        self::server::extension(address.to_string(), port.clone())
      }
      Self::CustomRuntimeModule(specifier, code) => BuiltinExtension {
        runtime_modules: vec![(specifier, code)],
        ..Default::default()
      },
      Self::Custom(ext) => ext(),
    }
  }
}

#[derive(Default)]
pub struct BuiltinExtensions {
  extensions: Vec<BuiltinExtension>,
}

impl BuiltinExtensions {
  pub fn load_all_snapshot_modules(runtime: &mut JsRuntime) {
    for extension in Self::all_builtin_extensions().iter() {
      for module in &extension.snapshot_modules {
        futures::executor::block_on(async {
          let mod_id = runtime
            .load_side_module(
              &Url::parse(&format!("builtin:///{}", module.0))?,
              Some(
                ExtensionFileSourceCode::LoadedFromFsDuringSnapshot(
                  module.1.clone(),
                )
                .load()?,
              ),
            )
            .await?;
          let receiver = runtime.mod_evaluate(mod_id);
          runtime.run_event_loop(false).await?;
          receiver.await?
        })
        .unwrap();
      }
    }
  }

  pub fn with_modules(modules: Vec<BuiltinModule>) -> Self {
    let extensions = modules
      .iter()
      .map(|m| m.extension())
      .collect::<Vec<BuiltinExtension>>();
    Self { extensions }
  }

  pub fn load_runtime_modules(&self, runtime: &mut JsRuntime) -> Result<()> {
    for extension in self.extensions.iter() {
      for module in &extension.runtime_modules {
        let (specifier, code) = module;
        futures::executor::block_on(async {
          debug!("Loading built-in module into the runtime: {}", specifier);
          let mod_id = runtime
            .load_side_module(
              &Url::parse(&format!("builtin:///{}", specifier))?,
              Some(ExtensionFileSourceCode::IncludedInBinary(code).load()?),
            )
            .await?;
          let receiver = runtime.mod_evaluate(mod_id);
          runtime.run_event_loop(false).await?;
          receiver.await?
        })?;
      }
    }
    Ok(())
  }

  pub fn add_module(&mut self, module: BuiltinModule) {
    self.extensions.push(module.extension())
  }

  pub fn deno_extensions(&mut self) -> Vec<Extension> {
    self
      .extensions
      .iter_mut()
      .map(|e| e.extension.take())
      .filter(|e| e.is_some())
      .map(|e| e.unwrap())
      .collect()
  }

  pub fn get_specifiers(&self) -> IndexSet<String> {
    self
      .extensions
      .iter()
      .map(|e| {
        let snapshot_modules: &Vec<(&str, PathBuf)> =
          e.snapshot_modules.as_ref();
        let runtime_modules: &Vec<(&str, &str)> = e.runtime_modules.as_ref();
        vec![
          snapshot_modules
            .iter()
            .map(|m| m.0.to_string())
            .collect::<Vec<String>>(),
          runtime_modules
            .iter()
            .map(|m| m.0.to_string())
            .collect::<Vec<String>>(),
        ]
        .concat()
      })
      .flatten()
      .collect::<IndexSet<String>>()
  }

  fn all_builtin_extensions() -> Vec<BuiltinExtension> {
    vec![
      BuiltinModule::Fs,
      BuiltinModule::Node,
      BuiltinModule::Postgres,
      BuiltinModule::Resolver(PathBuf::default()),
      BuiltinModule::Transpiler,
      BuiltinModule::Babel,
      BuiltinModule::Rollup,
      BuiltinModule::HttpServer("", 0),
    ]
    .iter()
    .map(|m| m.extension())
    .collect()
  }
}
