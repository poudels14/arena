use self::request::HttpRequest;
use self::resources::{
  HttpConnection, HttpResponseHandle, HttpServer, HttpServerConfig,
};
use super::extension::BuiltinExtension;
use crate::resolve_from_root;
use anyhow::{anyhow, Result};
use deno_core::CancelFuture;
use deno_core::{
  op, ByteString, CancelHandle, Extension, OpState, ResourceId, StringOrBuffer,
};
use futures::future::{pending, select, Either};
use futures::never::Never;
use futures::FutureExt;
use http::header::HeaderName;
use http::{HeaderValue, Method, Response};
use hyper::{server::conn::Http, Body};
use std::cell::RefCell;
use std::error::Error;
use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::pin::pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::task::spawn_local;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
mod executor;
mod request;
mod resources;

pub fn extension(option: (String, u16)) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init(option)),
    runtime_modules: vec![],
    snapshot_modules: vec![(
      "@arena/runtime/server",
      resolve_from_root!("../../js/arena-runtime/dist/server.js"),
    )],
  }
}

/// initialize server extension with given (address, port)
pub(crate) fn init(option: (String, u16)) -> Extension {
  Extension::builder("arena/runtime/server")
    .ops(vec![
      op_http_listen::decl(),
      op_http_accept::decl(),
      op_http_start::decl(),
      op_http_send_response::decl(),
    ])
    .state(move |state| {
      state.put::<HttpServerConfig>(HttpServerConfig {
        address: option.0.clone(),
        port: option.1,
      });
    })
    .build()
}

#[op]
async fn op_http_listen(state: Rc<RefCell<OpState>>) -> Result<()> {
  let config = state.borrow().borrow::<HttpServerConfig>().clone();
  let addr: SocketAddr =
    (Ipv4Addr::from_str(&config.address)?, config.port).into();
  let tcp_listener = TcpListener::bind(addr).await?;

  state.borrow_mut().put::<HttpServer>(HttpServer {
    address: config.address,
    port: config.port,
    listener: Rc::new(RefCell::new(tcp_listener)),
  });

  Ok(())
}

#[op]
async fn op_http_accept(state: Rc<RefCell<OpState>>) -> Result<ResourceId> {
  let server = {
    let state = state.borrow();
    state.borrow::<HttpServer>().clone()
  };

  let (tcp_stream, _) = server.listener.borrow_mut().accept().await?;
  let (tx, rx) =
    mpsc::channel::<(HttpRequest, mpsc::Sender<Response<Body>>)>(10);

  let cors = CorsLayer::new()
    .allow_methods([Method::GET])
    .allow_origin(AllowOrigin::list(vec![]));

  let service = ServiceBuilder::new()
    .layer(CompressionLayer::new())
    .layer(cors)
    .service_fn(move |req| {
      request::handle_request(
        server.address.clone(),
        server.port,
        tx.clone(),
        req,
      )
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
        closed_fut,
      });

  Ok(connection_rid)
}

#[op]
async fn op_http_start(
  state: Rc<RefCell<OpState>>,
  rid: u32,
) -> Result<Option<(ResourceId, HttpRequest)>> {
  let connection = state.borrow().resource_table.get::<HttpConnection>(rid)?;
  let mut stream = connection.req_stream.borrow_mut();

  match stream.recv().await {
    Some((req, resp)) => {
      let response_handle = state
        .borrow_mut()
        .resource_table
        .add::<HttpResponseHandle>(HttpResponseHandle { sender: resp });
      return Ok(Some((response_handle, req)));
    }
    None => return Ok(None),
  }
}

#[op]
async fn op_http_send_response(
  state: Rc<RefCell<OpState>>,
  rid: u32,
  status: u16,
  headers: Vec<(ByteString, ByteString)>,
  data: Option<StringOrBuffer>,
) -> Result<()> {
  let handle = state
    .borrow()
    .resource_table
    .get::<HttpResponseHandle>(rid)?;

  let mut response_builder = Response::builder().status(status);
  for header in headers {
    response_builder = response_builder.header(
      HeaderName::from_bytes(&header.0)?,
      HeaderValue::from_bytes(&header.1)?,
    );
  }

  let response = response_builder.body(Body::from(
    <StringOrBuffer as Into<bytes::Bytes>>::into(
      data.unwrap_or(StringOrBuffer::String("".to_owned())),
    )
    .slice(0..),
  ))?;
  handle
    .sender
    .send(response)
    .await
    .map_err(|e| anyhow!("{:?}", e))
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
