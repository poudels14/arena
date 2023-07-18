mod db;
mod search;
mod utils;
mod vectors;

use anyhow::{anyhow, Result};
use db::{DatabaseOptions, VectorDatabase};
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pythonize::{depythonize, pythonize};

#[pyclass]
struct Database {
  path: String,
  db: Option<VectorDatabase>,
}

#[pymethods]
impl Database {
  #[new]
  fn new(path: &str) -> PyResult<Self> {
    Ok(Self {
      path: path.to_string(),
      db: Some(VectorDatabase::open(
        path,
        DatabaseOptions {
          enable_statistics: true,
        },
      )?),
    })
  }

  fn create_collection<'a>(
    &mut self,
    py: Python<'a>,
    id: &str,
    collection: PyObject,
  ) -> PyResult<()> {
    let col = depythonize(collection.as_ref(py))?;
    Ok(self.get_mut_db()?.create_collection(id.into(), col)?)
  }

  fn list_collections<'a>(&self, py: Python<'a>) -> PyResult<Py<PyAny>> {
    let collections = self.get_db()?.list_collections()?;
    Ok(pythonize(py, &collections)?)
  }

  fn get_collection<'a>(
    &self,
    py: Python<'a>,
    id: &str,
  ) -> PyResult<Py<PyAny>> {
    let col = self.get_db()?.get_collection(id.into())?;
    Ok(pythonize(py, &col)?)
  }

  pub fn add_document<'a>(
    &mut self,
    py: Python<'a>,
    collection_id: &str,
    doc_id: &str,
    document: PyObject,
  ) -> PyResult<()> {
    let document = depythonize(document.as_ref(py))?;
    self.get_mut_db()?.add_document(
      collection_id.into(),
      doc_id.into(),
      document,
    )?;
    Ok(())
  }

  fn list_documents<'a>(
    &self,
    py: Python<'a>,
    collection_id: &str,
  ) -> PyResult<Py<PyAny>> {
    let documents = self.get_db()?.list_documents(collection_id.into())?;
    Ok(pythonize(py, &documents)?)
  }

  fn get_document<'a>(
    &'a self,
    py: Python<'a>,
    collection_id: &str,
    document_id: &str,
  ) -> PyResult<Option<Py<PyAny>>> {
    let document = self
      .get_db()?
      .get_document(collection_id.into(), document_id.into())?;
    Ok(document.map(|doc| pythonize(py, &doc)).transpose()?)
  }

  fn get_document_content<'a>(
    &'a self,
    py: Python<'a>,
    collection_id: &str,
    document_id: &str,
  ) -> PyResult<Option<&PyBytes>> {
    Ok(
      self
        .get_db()?
        .get_document_content_handle(collection_id.into())?
        .get_pinned_slice(document_id.into())?
        .map(|c| PyBytes::new(py, &c)),
    )
  }

  fn set_document_embeddings<'a>(
    &mut self,
    py: Python<'a>,
    collection_id: &str,
    doc_id: &str,
    chunks: PyObject,
  ) -> PyResult<()> {
    let chunks = depythonize(chunks.as_ref(py))?;
    Ok(self.get_mut_db()?.set_document_embeddings(
      collection_id.into(),
      doc_id.into(),
      chunks,
    )?)
  }

  fn scan_embeddings<'a>(
    &self,
    py: Python<'a>,
    collection_id: &str,
  ) -> PyResult<Py<PyAny>> {
    let embeddings = self.get_db()?.scan_embeddings(collection_id.into())?;
    Ok(pythonize(py, &embeddings)?)
  }

  fn search_collection<'a>(
    &self,
    py: Python<'a>,
    collection_id: &str,
    query: Vec<f32>,
    k: usize,
  ) -> PyResult<Py<PyAny>> {
    let s = search::FsSearch::using(self.get_db()?);
    let result = s.top_k(collection_id.into(), &query, k)?;
    Ok(pythonize(py, &result)?)
  }

  fn close(&mut self) -> PyResult<()> {
    if self.db.is_some() {
      let mut db = self.db.take().unwrap();
      db.close()?;
      return Ok(());
    }
    Err(anyhow!("Database already closed").into())
  }

  fn destroy(&self) -> PyResult<()> {
    VectorDatabase::destroy(&self.path)?;
    Ok(())
  }
}

impl Database {
  fn get_db<'a>(&'a self) -> Result<&'a VectorDatabase> {
    self
      .db
      .as_ref()
      .ok_or(anyhow!("Database already closed").into())
  }

  fn get_mut_db<'a>(&'a mut self) -> Result<&'a mut VectorDatabase> {
    self
      .db
      .as_mut()
      .ok_or(anyhow!("Database already closed").into())
  }
}

/// A Python module implemented in Rust.
#[pymodule]
fn vectordb(_: Python<'_>, m: &PyModule) -> PyResult<()> {
  m.add_class::<Database>()?;

  Ok(())
}
