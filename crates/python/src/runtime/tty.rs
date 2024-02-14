use bytes::{BufMut, BytesMut};
use pyo3::prelude::*;

#[pyclass]
pub struct TtyBuffer {
  pub buf: BytesMut,
}

impl TtyBuffer {
  pub fn new() -> Self {
    let buf = BytesMut::with_capacity(1024);
    Self { buf }
  }
}

#[pymethods]
impl TtyBuffer {
  fn write(&mut self, data: &str) {
    self.buf.put(data.as_bytes());
  }

  fn serialize(&self) -> PyResult<String> {
    Ok(std::str::from_utf8(&self.buf[..])?.to_string().into())
  }
}
