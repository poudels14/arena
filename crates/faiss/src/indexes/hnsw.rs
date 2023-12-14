use std::path::Path;

use getset::{Getters, Setters};
use strum_macros::Display;

use super::index::{my_index_factory, BaseIndex, Index};
use crate::error::Result;
use crate::faiss_ffi::{faiss_Index_reconstruct_n, faiss_try};
use crate::{MetricType, SearchResult, VectorId};

pub struct HNSWIndex {
  base: BaseIndex,
}

#[derive(Getters, Setters)]
pub struct IndexOptions {
  #[getset(get = "pub", set = "pub")]
  m: u16,

  #[getset(get = "pub", set = "pub")]
  ef_construction: u64,

  /// Defaults to true
  #[getset(get = "pub", set = "pub")]
  support_vector_ids: bool,

  /// Supported: Flat | "_PQ" | "_SQ4" | "_SQ8" | "SQ6" | "SQfp16"
  #[getset(get = "pub", set = "pub")]
  variant: String,
}

impl Default for IndexOptions {
  fn default() -> Self {
    Self {
      m: 4,
      ef_construction: 10,
      support_vector_ids: true,
      variant: "Flat".to_owned(),
    }
  }
}

#[derive(Display, Debug, Clone)]
pub enum Parameter {
  #[strum(serialize = "efConstruction")]
  EfConstruction,
}

impl HNSWIndex {
  pub fn new(
    d: u32,
    metric_type: MetricType,
    options: IndexOptions,
  ) -> Result<Self> {
    let mut description = "".to_owned();
    if options.support_vector_ids {
      description += "IDMap,"
    }
    description += &format!("HNSW{},{}", options.m, options.variant);
    let mut index = my_index_factory(d, &description, metric_type)?;
    index.set_parameter("efConstruction", options.ef_construction as f64);

    Ok(Self { base: index })
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
    let vectors = Vec::<f32>::with_capacity(self.base.d() as usize * 10);
    unsafe {
      faiss_try(faiss_Index_reconstruct_n(
        self.base.inner,
        0,
        10,
        vectors.as_ptr() as *mut _,
      ))
      .unwrap();
    }
    println!("vectors = {:?}", vectors);
  }
}

impl Index for HNSWIndex {
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
