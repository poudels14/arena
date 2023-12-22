use std::cell::RefCell;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::{anyhow, bail, Error};
use deno_ast::MediaType;
use deno_core::{
  FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier,
  ModuleType, ResolutionKind,
};
use futures::future::FutureExt;
use tokio::sync::{mpsc, oneshot};
use url::Url;

use super::transpiler;
use crate::extensions::server::response::ParsedHttpResponse;
use crate::extensions::server::{HttpRequest, HttpServerConfig};
use crate::resolver::ModuleResolver;
use crate::resolver::Resolver;
use crate::{extensions, IsolatedRuntime, RuntimeOptions};

pub struct FileModuleLoader {
  transpile: bool,
  transpiler_stream:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
  resolver: ModuleResolver,
}

pub struct ModuleLoaderOption {
  /// whether to auto-transpile the code when loading
  pub transpile: bool,
  pub resolver: Rc<dyn Resolver>,
}

#[cfg(feature = "build-tools")]
impl FileModuleLoader {
  pub fn new(option: ModuleLoaderOption) -> Self {
    let (stream_tx, stream_rx) = mpsc::channel(15);

    if option.transpile {
      // TODO(sagar): idk why doing this fixes segfault when creating another
      // TODO: remove me
      // runtime for transpiling in a new thread
      // let _ = IsolatedRuntime::new(RuntimeOptions {
      //   module_loader: Some(Rc::new(FileModuleLoader::new(
      //     ModuleLoaderOption {
      //       transpile: false,
      //       resolver: option.resolver.clone(),
      //     },
      //   ))),
      //   ..Default::default()
      // })
      // .unwrap();

      let resolver = option.resolver.clone();
      let local = tokio::task::LocalSet::new();
      local.spawn_local(async {
        let mut runtime = IsolatedRuntime::new(RuntimeOptions {
          enable_console: true,
          builtin_extensions: vec![extensions::server::extension(
            HttpServerConfig::Stream(Rc::new(RefCell::new(stream_rx))),
          )],
          module_loader: Some(Rc::new(FileModuleLoader::new(
            ModuleLoaderOption {
              transpile: false,
              resolver,
            },
          ))),
          ..Default::default()
        })
        .unwrap();

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
      });
    }

    Self {
      transpile: option.transpile,
      resolver: ModuleResolver::new(Some(option.resolver)),
      transpiler_stream: stream_tx,
    }
  }
}

// Note(sagar): copied from deno_core crate
// TODO(sagar): for some reason, this is being called more than once even
// for a single import, fix it?
impl ModuleLoader for FileModuleLoader {
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
        MediaType::Json => {
          (ModuleType::JavaScript, Some(self::load_json(&path)?), false)
        }
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

fn load_json(path: &PathBuf) -> Result<Arc<str>, Error> {
  let json = std::fs::read_to_string(path.clone())?;
  Ok(format!(r#"export default JSON.parse(`{json}`);"#).into())
}
