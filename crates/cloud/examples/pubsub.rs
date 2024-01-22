use std::rc::Rc;

use anyhow::Result;
use cloud::identity::Identity;
use cloud::pubsub::exchange::Exchange;
use cloud::pubsub::{EventSink, OutgoingEvent, Subscriber};
use cloud::CloudExtensionProvider;
use runtime::extensions::BuiltinExtensionProvider;
use runtime::extensions::BuiltinModule;
use runtime::{IsolatedRuntime, RuntimeOptions};
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
    config: Default::default(),
    builtin_extensions: builtin_modules
      .iter()
      .map(|m| m.get_extension())
      .collect(),
    enable_arena_global: true,
    ..Default::default()
  })?;

  let runtime = r.runtime.borrow_mut();
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
    true,
  )
  .await?;

  println!("DONE!");
  Ok(())
}
