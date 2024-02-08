use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, bail, Result};
use async_trait::async_trait;
use bytes::Bytes;
use http::Method;
use tokio::sync::{mpsc, oneshot};
use url::Url;

use super::ModuleTranspiler;
use crate::config::node::ResolverConfig;
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
  pub async fn new(config: ResolverConfig) -> Self {
    let (transpiler_stream, stream_rx) = mpsc::channel(15);

    let (tx, rx) = oneshot::channel::<bool>();
    deno_unsync::spawn(async move {
      let mut runtime = IsolatedRuntime::new(RuntimeOptions {
        enable_console: true,
        builtin_extensions: vec![
          BuiltinModule::Node(None),
          BuiltinModule::Resolver(config),
          BuiltinModule::Babel,
          BuiltinModule::HttpServer(HttpServerConfig::Stream(Arc::new(
            Mutex::new(Some(stream_rx)),
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
          import { babel, presets } from "@arena/runtime/babel";
          import { serve } from "@arena/runtime/server";
          serve({
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
          false,
        )
        .await
        .expect("Error running babel transpiler");

      tx.send(true)
        .expect("Error sending babel transpiler ready notif");
      runtime
        .run_event_loop()
        .await
        .expect("Error running babel transpiler");
    });

    let _ = rx.await.expect("Failed to wait for babel transpiler");
    Self { transpiler_stream }
  }
}

#[async_trait]
impl ModuleTranspiler for BabelTranspiler {
  async fn transpile(&self, _path: &PathBuf, code: &str) -> Result<Arc<str>> {
    let stream = self.transpiler_stream.clone();
    let (tx, rx) = oneshot::channel();
    stream.send(((Method::POST, code).into(), tx)).await?;
    if let Ok(response) = rx.await {
      return response
        .data
        .and_then(|b| Some(Bytes::from(b).to_vec()))
        .and_then(|v| {
          Some(
            simdutf8::basic::from_utf8(&v)
              .expect("Error reading transpiled code")
              .to_owned()
              .into(),
          )
        })
        .ok_or(anyhow!("Error reading transpiled code"));
    }
    bail!("Failed to transpile code using babel");
  }
}
