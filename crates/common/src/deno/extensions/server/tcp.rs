pub use self::request::HttpRequest;
use super::errors;
use super::request::HandleOptions;
use super::resonse::{HttpResponse, HttpResponseMetata};
use super::resources::{HttpConnection, HttpServerConfig, TcpServer};
use super::{executor, request};
use anyhow::Result;
use deno_core::CancelFuture;
use deno_core::{op, CancelHandle, OpState, ResourceId};
use futures::future::{pending, select, Either};
use futures::never::Never;
use futures::FutureExt;
use http::Method;
use hyper::server::conn::Http;
use std::cell::RefCell;
use std::error::Error;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::pin::pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::spawn_local;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tracing::info;

#[op]
pub(crate) async fn op_http_listen(state: Rc<RefCell<OpState>>) -> Result<()> {
  let config = state.borrow().borrow::<HttpServerConfig>().clone();

  match config {
    HttpServerConfig::Tcp {
      address,
      port,
      serve_dir,
    } => {
      let addr: SocketAddr = (Ipv4Addr::from_str(&address)?, port).into();
      let listener = TcpListener::bind(addr).await?;

      state.borrow_mut().put::<TcpServer>(TcpServer {
        listener: Rc::new(RefCell::new(listener)),
        serve_dir,
      });
      Ok(())
    }
    _ => unreachable!(),
  }
}

#[op]
pub(crate) async fn op_http_accept(
  state: Rc<RefCell<OpState>>,
) -> Result<ResourceId> {
  let server = {
    let state = state.borrow();
    state.borrow::<TcpServer>().clone()
  };

  let (tcp_stream, _) = server.listener.borrow_mut().accept().await?;
  let (tx, rx) = mpsc::channel::<(HttpRequest, mpsc::Sender<HttpResponse>)>(10);

  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(AllowOrigin::list(vec![]));

  let handle_options = HandleOptions {
    serve_dir: server.serve_dir,
  };

  let service = ServiceBuilder::new()
    .layer(CompressionLayer::new())
    .layer(cors)
    .map_result(
      |res: Result<(HttpResponse, HttpResponseMetata), errors::Error>| {
        <Result<HttpResponse, errors::Error>>::Ok(
          res
            .map(|res| {
              info!(
                "{} {:?} {} {}",
                res.1.method,
                res.1.path,
                res.0.status().as_u16(),
                format!(
                  "{}ms",
                  Instant::now()
                    .duration_since(res.1.req_received_at)
                    .as_millis()
                )
              );
              res.0
            })
            .map_err(|err| {
              tracing::error_span!("request", error = err.to_string());
              err
            })
            .unwrap_or(errors::internal_server_error()),
        )
      },
    )
    .service_fn(move |req| {
      request::handle_request(tx.clone(), handle_options.clone(), req)
    });

  let conn_fut = Http::new()
    .with_executor(executor::LocalExecutor)
    .http1_keep_alive(true)
    .serve_connection(tcp_stream, service);

  let cancel_handle = CancelHandle::new_rc();
  let shutdown_fut = pending::<Never>().or_cancel(&cancel_handle).fuse();

  // A local task that polls the hyper connection future to completion.
  let task_fut = async move {
    let conn_fut = std::pin::pin!(conn_fut);
    let shutdown_fut = pin!(shutdown_fut);
    let result = match select(conn_fut, shutdown_fut).await {
      Either::Left((result, _)) => result,
      Either::Right((_, mut conn_fut)) => {
        conn_fut.as_mut().graceful_shutdown();
        conn_fut.await
      }
    };

    filter_enotconn(result).map_err(Arc::from)
  };
  let (task_fut, closed_fut) = task_fut.remote_handle();
  let closed_fut = closed_fut.shared();
  spawn_local(task_fut);

  let connection_rid =
    state
      .borrow_mut()
      .resource_table
      .add::<HttpConnection>(HttpConnection {
        req_stream: Rc::new(RefCell::new(rx)),
        // TODO(sagar): properly close the tcp stream
        // handler.closed_fut.clone().map_err(AnyError::from).await?;
        closed_fut: Some(closed_fut),
      });
  Ok(connection_rid)
}

/// Filters out the ever-surprising 'shutdown ENOTCONN' errors.
fn filter_enotconn(
  result: Result<(), hyper::Error>,
) -> Result<(), hyper::Error> {
  if result
    .as_ref()
    .err()
    .and_then(|err| err.source())
    .and_then(|err| err.downcast_ref::<std::io::Error>())
    .filter(|err| err.kind() == std::io::ErrorKind::NotConnected)
    .is_some()
  {
    Ok(())
  } else {
    result
  }
}
