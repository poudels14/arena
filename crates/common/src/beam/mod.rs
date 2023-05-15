use anyhow::anyhow;
use anyhow::Result;
use std::fmt::Debug;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

#[derive(Clone, Debug)]
pub struct Sender<I, O>(mpsc::Sender<(I, oneshot::Sender<O>)>);

pub struct Receiver<I, O>(mpsc::Receiver<(I, oneshot::Sender<O>)>);

pub fn channel<I, O>(buffer: usize) -> (Sender<I, O>, Receiver<I, O>) {
  let (tx, rx) = mpsc::channel(buffer);
  (Sender(tx), Receiver(rx))
}

impl<I, O> Sender<I, O>
where
  I: Debug + Sync + Send + 'static,
  O: Debug + Sync + Send + 'static,
{
  pub async fn send(&self, value: I) -> Result<O> {
    let (tx, rx) = oneshot::channel::<O>();
    let _ = self.0.send((value, tx)).await?;
    rx.await.map_err(|e| anyhow!("{:?}", e))
  }
}

impl<I, O> Receiver<I, O>
where
  I: Debug,
{
  pub async fn recv(&mut self) -> Option<(I, oneshot::Sender<O>)> {
    self.0.recv().await
  }
}
