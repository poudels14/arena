use anyhow::Result;
use pyo3::types::PyDict;
use pyo3::{PyAny, Python};

pub struct SerializedResult {
  // "json" | "dataframe" | etc
  pub r#type: String,
  // JSON value
  pub value: String,
}

pub fn serialize_py_obj(
  py: &Python,
  ctxt: &PyDict,
  obj: &PyAny,
) -> Result<Option<SerializedResult>> {
  let portal = py.eval("portal.serde.serialize", Some(&ctxt), Some(&ctxt))?;
  let res: Option<Vec<String>> = portal.call((obj,), None)?.extract().unwrap();
  match res {
    Some(mut data) => Ok(Some(SerializedResult {
      value: data.pop().unwrap(),
      r#type: data.pop().unwrap(),
    })),
    None => Ok(None),
  }
}
