pub mod babel;
pub mod bundler;
pub mod env;
pub mod filerouter;
pub mod fs;
pub mod node;
pub mod postgres;
pub mod resolver;
pub mod rollup;
pub mod server;
pub mod sqlite;
pub mod transpiler;
pub mod wasi;

use self::server::HttpServerConfig;
use anyhow::{Context, Result};
use deno_core::{Extension, JsRuntime, ModuleCode};
use derivative::Derivative;
use indexmap::IndexSet;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::debug;
use url::Url;

#[derive(Default)]
pub struct BuiltinExtension {
  pub extension: Option<Extension>,
  /// tuples of module's (specifier, path_to_source_file)
  /// these modules are loaded during snapshoting
  pub snapshot_modules: Vec<(&'static str, PathBuf)>,

  /// tuples of module's (specifier, source_code)
  /// these modules are loaded during runtime
  pub runtime_modules: Vec<(&'static str, &'static str)>,
}

pub trait BuiltinExtensionProvider {
  fn get_extension(&self) -> BuiltinExtension;
}

#[derive(Clone, Derivative)]
#[derivative(Debug)]
pub enum BuiltinModule {
  Fs,
  Env,
  Node(Option<Vec<&'static str>>),
  Resolver(PathBuf),
  Transpiler,
  Babel,
  Rollup,
  Bundler,
  FileRouter,
  Postgres,
  Sqlite,
  HttpServer(HttpServerConfig),
  /// args: (specifier, code)
  CustomRuntimeModule(&'static str, &'static str),
  UsingProvider(
    #[derivative(Debug = "ignore")] Rc<dyn BuiltinExtensionProvider>,
  ),
  Custom(#[derivative(Debug = "ignore")] Rc<dyn Fn() -> BuiltinExtension>),
}

impl BuiltinModule {
  pub(crate) fn extension(&self) -> BuiltinExtension {
    match self {
      Self::Fs => self::fs::extension(),
      Self::Env => self::env::extension(),
      Self::Node(filter) => self::node::extension(filter.to_owned()),
      Self::Resolver(root) => self::resolver::extension(root.clone()),
      Self::Transpiler => self::transpiler::extension(),
      Self::Babel => self::babel::extension(),
      Self::Rollup => self::rollup::extension(),
      Self::Bundler => self::bundler::extension(),
      Self::FileRouter => self::filerouter::extension(),
      Self::Postgres => self::postgres::extension(),
      Self::Sqlite => self::sqlite::extension(),
      Self::HttpServer(config) => self::server::extension(config.clone()),
      Self::CustomRuntimeModule(specifier, code) => BuiltinExtension {
        runtime_modules: vec![(specifier, code)],
        ..Default::default()
      },
      Self::UsingProvider(p) => p.get_extension(),
      Self::Custom(ext) => ext(),
    }
  }
}

#[derive(Default)]
pub struct BuiltinExtensions {
  extensions: Vec<BuiltinExtension>,
}

impl BuiltinExtensions {
  pub fn with_all_modules() -> Self {
    let extensions = vec![
      BuiltinModule::Fs,
      BuiltinModule::Node(None),
      BuiltinModule::Postgres,
      BuiltinModule::Sqlite,
      BuiltinModule::Resolver(PathBuf::default()),
      BuiltinModule::Transpiler,
      BuiltinModule::Babel,
      BuiltinModule::Rollup,
      BuiltinModule::Bundler,
      BuiltinModule::FileRouter,
      BuiltinModule::HttpServer(HttpServerConfig::Tcp {
        address: "0.0.0.0".to_owned(),
        port: 0,
        serve_dir: None,
      }),
    ]
    .iter()
    .map(|m| m.extension())
    .collect::<Vec<BuiltinExtension>>();
    Self { extensions }
  }

  pub fn with_modules(modules: Vec<BuiltinModule>) -> Self {
    let extensions = modules
      .iter()
      .map(|m| m.extension())
      .collect::<Vec<BuiltinExtension>>();
    Self { extensions }
  }

  pub fn load_snapshot_modules(&self, runtime: &mut JsRuntime) -> Result<()> {
    for extension in self.extensions.iter() {
      for module in &extension.snapshot_modules {
        futures::executor::block_on(async {
          let msg = || format!("Failed to read \"{}\"", module.1.display());
          let s =
            std::fs::read_to_string(module.1.clone()).with_context(msg)?;

          let mod_id = runtime
            .load_side_module(
              &Url::parse(&format!("builtin://{}", module.0))?,
              Some(s.into()),
            )
            .await?;
          let receiver = runtime.mod_evaluate(mod_id);
          runtime.run_event_loop(false).await?;
          receiver.await?
        })?
      }
    }
    Ok(())
  }

  pub fn load_runtime_modules(&self, runtime: &mut JsRuntime) -> Result<()> {
    futures::executor::block_on(async {
      for extension in self.extensions.iter() {
        for module in &extension.runtime_modules {
          let (specifier, code) = module;
          debug!("Loading built-in module into the runtime: {}", specifier);
          let mod_id = runtime
            .load_side_module(
              &Url::parse(&format!("builtin://{}", specifier))?,
              ModuleCode::from_static(code).into(),
            )
            .await?;
          let receiver = runtime.mod_evaluate(mod_id);
          runtime.run_event_loop(false).await?;
          receiver.await??;
        }
      }
      Ok(())
    })
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
}
