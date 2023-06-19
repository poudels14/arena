use super::super::transpiler;
use crate::{IsolatedRuntime, RuntimeOptions};
use anyhow::{anyhow, bail, Error};
use common::config::ArenaConfig;
use common::deno::extensions::server::response::HttpResponse;
use common::deno::extensions::server::{HttpRequest, HttpServerConfig};
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use common::deno::resolver::fs::FsModuleResolver;
use deno_ast::MediaType;
use deno_core::{
  FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier,
  ModuleType, ResolutionKind,
};
use futures::future::FutureExt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use url::Url;

pub(crate) struct FsModuleLoader {
  transpile: bool,
  transpiler_stream: mpsc::Sender<(HttpRequest, mpsc::Sender<HttpResponse>)>,
  resolver: FsModuleResolver,
}

pub struct ModuleLoaderOption {
  /// whether to auto-transpile the code when loading
  pub transpile: bool,

  pub resolver: FsModuleResolver,
}

impl FsModuleLoader {
  pub fn new(option: ModuleLoaderOption) -> Self {
    let (stream_tx, stream_rx) = mpsc::channel(15);
    let project_root = option.resolver.project_root.clone();

    if option.transpile {
      thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
          .enable_all()
          .worker_threads(2)
          .max_blocking_threads(2)
          .build()
          .unwrap();

        let local = tokio::task::LocalSet::new();
        let _r = local.block_on(&rt, async {
          let mut runtime = IsolatedRuntime::new(RuntimeOptions {
            project_root: Some(project_root.clone()),
            config: Some(ArenaConfig::default()),
            enable_console: true,
            builtin_extensions: BuiltinExtensions::with_modules(vec![
              BuiltinModule::HttpServer(HttpServerConfig::Stream(Rc::new(
                RefCell::new(stream_rx),
              ))),
            ]),
            // Note(sagar): since rollup is loaded as side-module when build
            // tools is enabled and rollup needs node modules,
            // need to enable module loader
            disable_module_loader: false,
            transpile: false,
            ..Default::default()
          }).unwrap();

          let local = tokio::task::LocalSet::new();
          local
            .run_until(async move {
            runtime
              .execute_main_module_code(
                &Url::parse("file:///main").unwrap(),
                r#"
                import { babel, plugins, presets } from "@arena/runtime/babel";
                import { serve } from "@arena/runtime/server";
                await serve({
                  async fetch(req) {
                    const code = await req.text();
                    const { code: transpiledCode } = babel.transform(code, {
                      presets: [
                        // Note(sagar): since the code transpiled here is only used in
                        // server side, it should be transpiled for "ssr"
                        [presets.solidjs, {
                          "generate": "ssr",
                          "hydratable": false,
                        }]
                      ],
                    });
                    return new Response(transpiledCode);
                  }
                });
                "#,
              )
              .await
              .unwrap();

              runtime.run_event_loop().await.unwrap();
            }).await;
        });
      });
    }

    Self {
      transpile: option.transpile,
      resolver: option.resolver,
      transpiler_stream: stream_tx,
    }
  }
}

// Note(sagar): copied from deno_core crate
// TODO(sagar): for some reason, this is being called more than once even
// for a single import, fix it?
impl ModuleLoader for FsModuleLoader {
  #[tracing::instrument(skip(self, _kind), level = "debug")]
  fn resolve(
    &self,
    specifier: &str,
    referrer: &str,
    _kind: ResolutionKind,
  ) -> Result<ModuleSpecifier, Error> {
    Ok(self.resolver.resolve(&specifier, referrer)?)
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<&ModuleSpecifier>,
    _is_dynamic: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let module_specifier = module_specifier.clone();

    let transpile = self.transpile;
    let transpiler_stream = self.transpiler_stream.clone();
    async move {
      let path = module_specifier.to_file_path().map_err(|_| {
        anyhow!(
          "Provided module specifier \"{}\" is not a file URL.",
          module_specifier
        )
      })?;

      let media_type = MediaType::from_specifier(&module_specifier);
      let (module_type, maybe_code, _should_transpile) = match media_type {
        MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
          (ModuleType::JavaScript, None, false)
        }
        MediaType::TypeScript
        | MediaType::Mts
        | MediaType::Cts
        | MediaType::Dts
        | MediaType::Dmts
        | MediaType::Dcts
        | MediaType::Tsx
        | MediaType::Jsx => (ModuleType::JavaScript, None, transpile),
        MediaType::Json => (ModuleType::Json, None, false),
        _ => match path.extension().and_then(|e| e.to_str()) {
          Some("css") => {
            (ModuleType::JavaScript, Some(self::load_css(&path)?), false)
          }
          _ => bail!("Unknown extension of path: {:?}", path),
        },
      };

      let code = match maybe_code {
        Some(code) => code,
        None => {
          let code = std::fs::read_to_string(path.clone())?;
          // Note(sagar): transpile all JS files if transpile is enabled
          // so that even cjs modules are transformed to es6
          match transpile {
            true => {
              transpiler::transpile(
                transpiler_stream,
                &path,
                &media_type,
                &code,
              )
              .await?
            }
            false => code.into(),
          }
        }
      };

      let module = ModuleSource::new(
        module_type,
        FastString::Arc(code.into()),
        &module_specifier,
      );
      Ok(module)
    }
    .boxed_local()
  }
}

fn load_css(path: &PathBuf) -> Result<Arc<str>, Error> {
  let css = std::fs::read_to_string(path.clone())?;
  Ok(format!(r#"export default `{css}`;"#).into())
}
