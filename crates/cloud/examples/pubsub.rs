use std::rc::Rc;

use anyhow::Result;
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::pubsub::{EventSink, OutgoingEvent, Subscriber};
use cloud::CloudExtensionProvider;
use common::deno::extensions::{BuiltinExtensions, BuiltinModule};
use jsruntime::{IsolatedRuntime, RuntimeOptions};
use tokio::sync::mpsc;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
  let exchange = Exchange::new("workspace_id".to_owned());
  let provider = CloudExtensionProvider {
    publisher: Some(
      exchange
        .new_publisher(Identity::User {
          id: "test_user".to_owned(),
        })
        .await,
    ),
  };

  tokio::spawn(async move {
    let (tx, mut rx) = mpsc::channel::<Vec<OutgoingEvent>>(10);
    let _ = exchange
      .add_subscriber(Subscriber {
        id: "0".into(),
        identity: Identity::Unknown,
        out_stream: EventSink::Stream(tx),
        filter: Default::default(),
      })
      .await;

    tokio::spawn(async move {
      let _ = exchange.run().await;
    });

    while let Some(e) = rx.recv().await {
      println!("EVENT RECEIVED: {:?}", e);
    }
  });

  let builtin_modules = vec![BuiltinModule::UsingProvider(Rc::new(provider))];

  let mut r = IsolatedRuntime::new(RuntimeOptions {
    enable_console: true,
    project_root: Some(std::env::current_dir().unwrap()),
    config: Some(Default::default()),
    builtin_extensions: BuiltinExtensions::with_modules(
      builtin_modules.clone(),
    ),
    enable_arena_global: true,
    ..Default::default()
  })?;

  let mut runtime = r.runtime.borrow_mut();
  BuiltinExtensions::with_modules(builtin_modules)
    .load_snapshot_modules(&mut runtime)?;
  drop(runtime);

  r.execute_main_module_code(
    &Url::parse("file:///main")?,
    r#"
    import { publish } from "@arena/cloud/pubsub";

    for (let i = 0; i < 10; i++) {
      publish({
        message: `hello there! [count = ${i}]`
      });
    }
    "#,
  )
  .await?;

  println!("DONE!");
  Ok(())
}
