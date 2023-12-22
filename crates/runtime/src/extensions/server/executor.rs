use futures::Future;

// Local excutor for hyper to use non Send futures
#[derive(Clone)]
pub struct LocalExecutor;

impl<Fut> hyper::rt::Executor<Fut> for LocalExecutor
where
  Fut: Future + 'static,
  Fut::Output: 'static,
{
  fn execute(&self, fut: Fut) {
    deno_unsync::spawn(fut);
  }
}
