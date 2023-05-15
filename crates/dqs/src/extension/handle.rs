use crate::server::Command;
use anyhow::Result;
use common::beam;
use deno_core::futures::FutureExt;
use deno_core::v8::IsolateHandle;
use deno_core::{AsyncResult, Resource};
use serde_json::Value;
use std::rc::Rc;
use std::thread::JoinHandle;

pub struct DqsServerHandle {
  pub thread_handle: JoinHandle<Result<()>>,
  pub isolate_handle: IsolateHandle,
  pub commands: beam::Sender<Command, Value>,
}

impl Resource for DqsServerHandle {
  fn close(self: Rc<Self>) {
    drop(self);
  }

  fn shutdown(self: Rc<Self>) -> AsyncResult<()> {
    let commands = self.commands.clone();
    async move {
      commands.send(Command::Terminate).await.map(|_| ())?;
      self.close();
      Ok(())
    }
    .boxed_local()
  }
}
