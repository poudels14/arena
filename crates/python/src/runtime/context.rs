use anyhow::Result;
use derivative::Derivative;
use pyo3::types::{IntoPyDict, PyDict, PyModule};
use pyo3::{IntoPy, PyAny, Python};

use super::serialize::SerializedResult;
use super::tty::TtyBuffer;
use crate::portal;
use crate::runtime::serialize::serialize_py_obj;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Context<'py> {
  #[derivative(Debug = "ignore")]
  py: Python<'py>,
  #[derivative(Debug = "ignore")]
  globals: &'py PyDict,
}

impl<'py> Context<'py> {
  pub fn new(py: Python<'py>) -> Result<Self> {
    let module = PyModule::new(py, "portal")?;
    portal::init(py, module)?;
    let module: &PyAny = module;
    let globals = [
      ("portal", module),
      ("display", module.getattr("serde")?.getattr("display")?),
    ]
    .into_py_dict(py);

    let sys = py.import("sys")?;
    let stdout = TtyBuffer::new();
    let stderr = TtyBuffer::new();
    sys.setattr("stdout", stdout.into_py(py))?;
    sys.setattr("stderr", stderr.into_py(py))?;
    Ok(Self { py, globals })
  }

  #[tracing::instrument(level = "trace")]
  pub fn exec(&self, code: &str) -> Result<Option<SerializedResult>> {
    let pop_last_expr = self
      .globals
      .get_item("portal")?
      .unwrap()
      .getattr("ast")?
      .getattr("pop_last_expr")?;

    let code_blocks: Vec<String> =
      pop_last_expr.call((code,), None)?.extract()?;

    self
      .py
      .run(&code_blocks[0], Some(&self.globals), Some(&self.globals))?;

    match code_blocks.get(1) {
      Some(stmt) => {
        let res = self
          .py
          .eval(&stmt, Some(&self.globals), Some(&self.globals))
          .unwrap();
        Ok(serialize_py_obj(&self.py, &self.globals, res).unwrap_or_default())
      }
      _ => Ok(None),
    }
  }

  pub fn stdout(&self) -> Result<String> {
    let sys = self.py.import("sys")?;
    let stdout = sys
      .getattr("stdout")?
      .getattr("serialize")?
      .call((), None)?
      .extract::<String>()?;
    Ok(stdout)
  }

  pub fn stderr(&self) -> Result<String> {
    let sys = self.py.import("sys")?;
    let stderr = sys
      .getattr("stderr")?
      .getattr("serialize")?
      .call((), None)?
      .extract::<String>()?;
    Ok(stderr)
  }
}
