use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use deno_ast::MediaType;
use tokio::sync::{mpsc, oneshot};
use url::Url;

use super::{transpiler, ModuleTranspiler};
use crate::extensions::server::response::ParsedHttpResponse;
use crate::extensions::server::{HttpRequest, HttpServerConfig};
use crate::extensions::{BuiltinExtensionProvider, BuiltinModule};
use crate::{IsolatedRuntime, RuntimeOptions};

pub struct BabelTranspiler {
  transpiler_stream:
    mpsc::Sender<(HttpRequest, oneshot::Sender<ParsedHttpResponse>)>,
}

impl BabelTranspiler {
  // TODO: pass in BUILD tools snapshot
  pub fn new(root_dir: PathBuf) -> Self {
    let (transpiler_stream, stream_rx) = mpsc::channel(15);

    deno_unsync::spawn(async {
      let mut runtime = IsolatedRuntime::new(RuntimeOptions {
        enable_console: true,
        builtin_extensions: vec![
          BuiltinModule::Node(None),
          BuiltinModule::Resolver(root_dir),
          BuiltinModule::Babel,
          BuiltinModule::HttpServer(HttpServerConfig::Stream(Rc::new(
            RefCell::new(stream_rx),
          ))),
        ]
        .iter()
        .map(|m| m.get_extension())
        .collect(),
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
        .expect("Error running babel transpiler");
      runtime
        .run_event_loop()
        .await
        .expect("Error running babel transpiler");
    });

    Self { transpiler_stream }
  }
}

#[async_trait]
impl ModuleTranspiler for BabelTranspiler {
  async fn transpile(
    &self,
    path: &PathBuf,
    media_type: &MediaType,
    code: &str,
  ) -> Result<Arc<str>> {
    let stream = self.transpiler_stream.clone();
    transpiler::transpile(stream, path, media_type, code).await
  }
}
