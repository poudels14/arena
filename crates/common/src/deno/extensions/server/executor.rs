use futures::Future;
use tokio::task::spawn_local;

// Local excutor for hyper to use non Send futures
#[derive(Clone)]
pub struct LocalExecutor;

impl<Fut> hyper::rt::Executor<Fut> for LocalExecutor
where
  Fut: Future + 'static,
  Fut::Output: 'static,
{
  fn execute(&self, fut: Fut) {
    spawn_local(fut);
  }
}
