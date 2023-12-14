use std::path::Path;

use getset::{Getters, Setters};
use strum_macros::Display;

use super::index::{my_index_factory, BaseIndex, Index};
use crate::error::Result;
use crate::faiss_ffi::{faiss_Index_reconstruct_n, faiss_try};
use crate::{MetricType, SearchResult, VectorId};

pub struct FlatIndex {
  base: BaseIndex,
}

#[allow(unused)]
#[derive(Getters, Setters)]
pub struct IndexOptions {
  #[getset(get, set)]
  m: u16,

  #[getset(get, set)]
  ef_construction: u64,

  with_ids: bool,

  /// Supported: Flat | "_PQ" | "_SQ4" | "_SQ8" | "SQ6" | "SQfp16"
  variant: String,
}

impl Default for IndexOptions {
  fn default() -> Self {
    Self {
      m: 4,
      ef_construction: 10,
      with_ids: true,
      variant: "Flat".to_owned(),
    }
  }
}

#[derive(Display, Debug, Clone)]
pub enum Parameter {
  #[strum(serialize = "efConstruction")]
  EfConstruction,
}

impl FlatIndex {
  pub fn new(
    d: u32,
    metric_type: MetricType,
    options: IndexOptions,
  ) -> Result<Self> {
    let mut description = "".to_owned();
    if options.with_ids {
      description += "IDMap2,"
    }
    description += &format!("Flat");
    let base = my_index_factory(d, &description, metric_type)?;
    Ok(Self { base })
  }

  #[inline]
  pub fn read_from_file(path: &Path) -> Result<Self> {
    Ok(Self {
      base: BaseIndex::read_from_file(path)?,
    })
  }

  pub fn set_parameter(&mut self, param: Parameter, value: f64) {
    self.base.set_parameter(&param.to_string(), value)
  }

  pub fn reconstruct_vectors(&self) {
    // TODO: I dont think this works
    let vectors = Vec::<f32>::with_capacity(self.base.d() as usize * 2);
    unsafe {
      faiss_try(faiss_Index_reconstruct_n(
        self.base.inner,
        0,
        2,
        vectors.as_ptr() as *mut _,
      ))
      .unwrap();
    }
    println!("vectors = {:?}", vectors);
  }
}

impl Index for FlatIndex {
  fn add(&mut self, x: &[f32]) -> Result<()> {
    self.base.add(x)
  }

  fn add_with_ids(&mut self, x: &[f32], xids: &[VectorId]) -> Result<()> {
    self.base.add_with_ids(x, xids)
  }

  fn search(&self, query: &[f32], k: usize) -> Result<SearchResult> {
    self.base.search(query, k)
  }

  fn search_with_parameter(&self) {
    self.base.search_with_parameter()
  }

  #[inline]
  fn write_to_file(&self, path: &Path) -> Result<()> {
    self.base.write_to_file(path)
  }
}
