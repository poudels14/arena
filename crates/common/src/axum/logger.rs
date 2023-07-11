use axum::middleware::Next;
use axum::response::Response;
use http::Request;
use tokio::time::Instant;

pub async fn middleware<B>(request: Request<B>, next: Next<B>) -> Response {
  let method = request.method().as_ref().to_string();
  let path = request.uri().path().to_string();
  let start = Instant::now();
  let res = next.run(request).await;

  println!(
    "{} {:?} {} {}",
    method,
    path,
    res.status().as_u16(),
    format!("{}ms", Instant::now().duration_since(start).as_millis())
  );
  res
}
