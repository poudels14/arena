use pyo3::prelude::*;

macro_rules! py_module {
  ($py:tt,$name:literal,$code:expr) => {
    Into::<Py<PyAny>>::into(PyModule::from_code($py, $code, "", $name)?)
  };
}

#[pymodule]
pub fn init(py: Python<'_>, m: &PyModule) -> PyResult<()> {
  m.add("version", env!("CARGO_PKG_VERSION"))?;
  m.add(
    "ast",
    py_module!(py, "portal.ast", include_str!("./ast.py")),
  )?;
  m.add(
    "serde",
    Into::<Py<PyAny>>::into(PyModule::from_code(
      py,
      include_str!("./serde.py"),
      "",
      "portal.serde",
    )?),
  )?;
  Ok(())
}
