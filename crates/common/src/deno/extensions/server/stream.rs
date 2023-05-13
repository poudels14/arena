use super::resources::{HttpConnection, StreamServer};
use super::HttpServerConfig;
use anyhow::Result;
use deno_core::{op, OpState, ResourceId};
use std::cell::RefCell;
use std::rc::Rc;

#[op]
pub(crate) async fn op_http_listen(state: Rc<RefCell<OpState>>) -> Result<()> {
  let config = { state.borrow().borrow::<HttpServerConfig>().clone() };
  let listener = match config {
    HttpServerConfig::Stream(listener) => listener,
    _ => unreachable!(),
  };

  state
    .borrow_mut()
    .put::<StreamServer>(StreamServer { listener });
  Ok(())
}

#[op]
pub(crate) async fn op_http_accept(
  state: Rc<RefCell<OpState>>,
) -> Result<Option<ResourceId>> {
  // Note(sagar): take the server from state and put it in the resource
  // table so that we can return None the second time http_accept is called
  // stream server will basically work as if there was only one tcp stream

  let server = state.borrow_mut().try_take::<StreamServer>();
  match server {
    Some(s) => {
      let req_stream = s.listener.clone();
      drop(s);

      Ok(Some(
        state.borrow_mut().resource_table.add::<HttpConnection>(
          HttpConnection {
            req_stream,
            closed_fut: None,
          },
        ),
      ))
    }
    None => Ok(None),
  }
}
