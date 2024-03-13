use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::pubsub::{EventSink, OutgoingEvent, Subscriber};
use deno_core::v8;
use dqs::arena::{ArenaRuntimeState, MainModule};
use dqs::loaders::{FileTemplateLoader, Registry};
use dqs::runtime::deno;
use runtime::extensions::server::HttpServerConfig;
use runtime::permissions::{
  FileSystemPermissions, NetPermissions, PermissionsContainer, TimerPermissions,
};
use tokio::sync::mpsc;

fn main() -> Result<()> {
  let v8_platform = v8::new_default_platform(0, false).make_shared();
  let start = Instant::now();

  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()?;

  let local = tokio::task::LocalSet::new();
  local.block_on(&rt, async {
    let (_, http_requests_rx) = mpsc::channel(5);
    let (tx, mut rx) = mpsc::channel::<Vec<OutgoingEvent>>(10);

    tokio::spawn(async move {
      while let Some(e) = rx.recv().await {
        println!("EVENT RECEIVED: {:?}", e);
      }
    });

    let main_module = MainModule::Inline {
      code: r#"
      import { publish, subscribe } from "@arena/cloud/pubsub";
      console.log("starting dqs...");

      // subscribe((event) => {
      //   console.log("RECIVED: ", event);
      // });

      setInterval(async () => {
        let result = await publish({
          message: "Hello world!",
        });
        // console.log("Published = ", result);
        console.log("UO!")
      }, 1_000);
      "#
      .to_owned(),
    };

    let exchange = Exchange::new("workspace_id".to_owned());

    let _ = exchange
      .add_subscriber(Subscriber {
        id: "0".into(),
        identity: Identity::User {
          id: "test_user".to_owned(),
        },
        out_stream: EventSink::Stream(tx),
        filter: Default::default(),
      })
      .await;

    let mut runtime = deno::new(deno::RuntimeOptions {
      id: "test_runtime".to_string(),
      db_pool: None,
      server_config: HttpServerConfig::Stream(Rc::new(RefCell::new(
        http_requests_rx,
      ))),
      permissions: Default::default(),
      // permissions: PermissionsContainer {
      //   fs: Some(FileSystemPermissions::allow_all("/".into())),
      //   net: Some(NetPermissions::allow_all()),
      //   timer: Some(TimerPermissions::allow_hrtime()),
      // },
      heap_limits: None,
      egress_address: None,
      v8_platform,
      exchange: Some(exchange.clone()),
      state: ArenaRuntimeState {
        workspace_id: "test_workspace".to_string(),
        module: main_module.clone(),
        env_variables: Default::default(),
      },
      template_loader: Arc::new(FileTemplateLoader {}),
    })
    .await?;

    tokio::spawn(async move {
      let _ = exchange.run().await;
    });

    let entry_module = main_module.get_entry_module()?;

    let mod_id = runtime
      .load_main_module(&entry_module.0, entry_module.1)
      .await?;

    let rx = runtime.mod_evaluate(mod_id);
    runtime.run_event_loop(Default::default()).await?;
    rx.await?;

    Ok::<(), anyhow::Error>(())
  })?;

  println!(
    "Time taken = {}",
    Instant::now().duration_since(start).as_millis()
  );

  Ok(())
}
