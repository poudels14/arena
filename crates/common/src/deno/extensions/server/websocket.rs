pub use self::request::HttpRequest;
use super::response::{HttpResponse, ParsedHttpResponse};
use super::{errors, request};
use anyhow::{anyhow, Result};
use deno_core::{op, OpState, ResourceId, ZeroCopyBuf};
use deno_core::{Resource, StringOrBuffer};
use digest::Digest;
use fastwebsockets::upgrade::upgrade;
use fastwebsockets::{FragmentCollector, Frame, OpCode, Payload};
use http::header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY};
use http::{HeaderName, HeaderValue, Request, Response, StatusCode};
use http_body::Empty;
use hyper::body::HttpBody;
use hyper::Body;
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::spawn_local;
use tracing::debug;

#[derive(Debug)]
pub struct WebsocketStream {
  pub rx: WebsocketIncomingStream,
  pub tx: WebsocketOutgoingStream,
}

#[derive(Debug)]
pub struct WebsocketIncomingStream(
  pub RefCell<mpsc::Receiver<WebsocketMessage>>,
);

impl Resource for WebsocketIncomingStream {
  fn close(self: Rc<Self>) {
    debug!("Incoming stream of websocket dropped");
    drop(self);
  }
}

impl From<mpsc::Receiver<WebsocketMessage>> for WebsocketIncomingStream {
  fn from(value: mpsc::Receiver<WebsocketMessage>) -> Self {
    Self(RefCell::new(value))
  }
}

#[derive(Debug)]
pub struct WebsocketOutgoingStream(pub mpsc::Sender<WebsocketMessage>);

impl Resource for WebsocketOutgoingStream {
  fn close(self: Rc<Self>) {
    debug!("Outgoing stream of websocket dropped");
    drop(self);
  }
}

impl From<mpsc::Sender<WebsocketMessage>> for WebsocketOutgoingStream {
  fn from(value: mpsc::Sender<WebsocketMessage>) -> Self {
    Self(value)
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct WebsocketMessage {
  pub payload: Option<StringOrBuffer>,
  pub close: bool,
}

#[op]
pub(crate) async fn op_websocket_recv(
  state: Rc<RefCell<OpState>>,
  receiver_rid: ResourceId,
  sender_rid: ResourceId,
) -> Result<Option<WebsocketMessage>> {
  let receiver = {
    state
      .borrow_mut()
      .resource_table
      .get::<WebsocketIncomingStream>(receiver_rid)?
  };

  let mut receiver = receiver.0.borrow_mut();
  match receiver.recv().await {
    Some(data) => Ok(Some(data)),
    None => {
      // None will be received if the mpsc sender is dropped
      // So, close the incoming stream and "close" both incoming/outgoing
      // streams
      state.borrow_mut().resource_table.close(receiver_rid)?;
      state.borrow_mut().resource_table.close(sender_rid)?;

      Ok(Some(WebsocketMessage {
        close: true,
        ..Default::default()
      }))
    }
  }
}

#[op]
pub(crate) async fn op_websocket_send(
  state: Rc<RefCell<OpState>>,
  sender_rid: ResourceId,
  value: WebsocketMessage,
) -> u16 {
  let sender = {
    state
      .borrow_mut()
      .resource_table
      .get::<WebsocketOutgoingStream>(sender_rid)
  };

  if let Ok(sender) = sender {
    // return 1 if message sending successful
    match sender.0.send(value).await {
      Ok(_) => return 1,
      _ => {}
    }
  }
  return 0;
}

pub fn upgrade_to_websocket(
  req: Request<Body>,
  res: ParsedHttpResponse,
) -> Result<HttpResponse, errors::Error> {
  let sec_accept = match req.headers().get(SEC_WEBSOCKET_KEY) {
    Some(sec_key) => sec_websocket_accept_header(sec_key.as_bytes()).ok(),
    None => None,
  };

  handle_websocket(req, res.websocket_tx);
  let mut response_builder = Response::builder()
    .status(StatusCode::SWITCHING_PROTOCOLS)
    .header(CONNECTION, HeaderValue::from_static("Upgrade"));
  for header in &res.headers {
    response_builder = response_builder.header(
      HeaderName::from_bytes(&header.0)?,
      HeaderValue::from_bytes(&header.1)?,
    );
  }
  if let Some(sec_accept) = sec_accept {
    response_builder = response_builder
      .header(SEC_WEBSOCKET_ACCEPT, HeaderValue::from_str(&sec_accept)?);
  }

  return Ok(
    response_builder
      .body(Empty::new().map_err(|_| unreachable!()).boxed_unsync())
      .unwrap(),
  );
}

pub(crate) fn sec_websocket_accept_header(key: &[u8]) -> Result<String> {
  let mut sha = Sha1::from(Default::default());
  sha
    .update(&[key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11".as_bytes()].concat());
  Ok(base64::encode(sha.finalize()))
}

pub fn handle_websocket(
  mut req: Request<Body>,
  websocket_tx: Option<oneshot::Sender<WebsocketStream>>,
) {
  spawn_local(async move {
    let (_response, fut) =
      upgrade(&mut req).expect("successful websocket upgrade");
    let mut ws = FragmentCollector::new(
      fut
        .await
        .map_err(|_| anyhow!("error upgrading to websocket"))?,
    );

    let (in_tx, in_rx) = mpsc::channel::<WebsocketMessage>(15);
    let (out_tx, mut out_rx) = mpsc::channel::<WebsocketMessage>(5);
    if let Some(websocket_tx) = websocket_tx {
      websocket_tx
        .send(WebsocketStream {
          rx: in_rx.into(),
          tx: out_tx.into(),
        })
        .expect("failed to send websocket stream");
    }

    loop {
      tokio::select! {
        frame = ws.read_frame() => {
          if let Ok(frame) = frame {
            match frame.opcode {
              OpCode::Close => {
                debug!("Websocket closed by the client");
                break;
              }
              OpCode::Text => {
                in_tx
                  .send(WebsocketMessage {
                    payload: Some(StringOrBuffer::String(
                      simdutf8::basic::from_utf8(&frame.payload)?.to_owned(),
                    )),
                    close: false,
                  })
                  .await
                  .map_err(|e| anyhow!("{}", e))?;
              }
              OpCode::Binary => {
                in_tx
                  .send(WebsocketMessage {
                    payload: Some(StringOrBuffer::Buffer(ZeroCopyBuf::ToV8(Some(
                      (&frame.payload).to_vec().into(),
                    )))),
                    close: false,
                  })
                  .await
                  .map_err(|e| anyhow!("{}", e))?;
              }
              OpCode::Ping => ws
                .write_frame(Frame::text(Payload::Owned("PONG".into())))
                .await
                .map_err(|_| anyhow!("error writing to websocket"))?,
              _ => {}
            }
          }
        },
        out_data = out_rx.recv() => {
          if let Some(msg) = out_data {
            let (opcode, payload) = msg
              .payload
              .map(|payload| match payload {
                StringOrBuffer::String(text) => {
                  (OpCode::Text, Payload::Owned(text.into()))
                }
                StringOrBuffer::Buffer(data) => {
                  (OpCode::Binary, Payload::Owned(data.to_vec()))
                }
              })
              .unwrap_or((OpCode::Continuation, Payload::Owned(vec![])));

            let frame = match msg.close {
              true => Frame::close(0, &payload),
              false => Frame::new(true, opcode, None, payload),
            };

            ws.write_frame(frame)
              .await
              .map_err(|_| anyhow!("error writing to websocket"))?;

            if msg.close {
              debug!("Websocket closed by the server");
              break;
            }
          }
        }
      }
    }
    drop(in_tx);
    drop(out_rx);
    Ok::<(), anyhow::Error>(())
  });
}
