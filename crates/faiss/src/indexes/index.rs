use std::ffi::{c_int, c_uint, CString};
use std::path::Path;

use crate::error::{Error, Result};
use crate::faiss_ffi::{
  self, faiss_Index_add, faiss_Index_add_with_ids, faiss_Index_d,
  faiss_Index_search, faiss_Index_set_verbose, faiss_idx_t, faiss_read_index,
  faiss_try, faiss_write_index,
};
use crate::metrics::MetricType;
use crate::parameters::ParameterSpace;
use crate::search::SearchResult;
use crate::vector::VectorId;

pub(crate) fn my_index_factory(
  d: u32,
  description: &str,
  metric: MetricType,
) -> Result<BaseIndex> {
  unsafe {
    let metric = metric as c_uint;
    let description = CString::new(description).unwrap();
    let mut index_ptr = ::std::ptr::null_mut();
    faiss_try(faiss_ffi::faiss_index_factory(
      &mut index_ptr,
      (d & 0x7FFF_FFFF) as i32,
      description.as_ptr(),
      metric,
    ))?;
    Ok(BaseIndex {
      inner: index_ptr,
      params: ParameterSpace::new()?,
    })
  }
}

#[derive(Debug)]
pub struct BaseIndex {
  pub(crate) inner: *mut faiss_ffi::FaissIndex,
  pub(crate) params: ParameterSpace,
}

pub trait Index {
  fn add(&mut self, x: &[f32]) -> Result<()>;

  fn add_with_ids(&mut self, x: &[f32], xids: &[VectorId]) -> Result<()>;

  fn search(&self, query: &[f32], k: usize) -> Result<SearchResult>;

  fn search_with_parameter(&self);

  fn write_to_file(&self, path: &Path) -> Result<()>;
}

#[allow(unused)]
impl BaseIndex {
  #[inline]
  pub fn set_parameter(&mut self, key: &str, value: f64) {
    unsafe {
      let param = key.to_string();
      faiss_ffi::faiss_ParameterSpace_set_index_parameter(
        self.params.inner,
        self.inner,
        param.as_ptr() as *const _,
        value,
      );
    }
  }

  #[inline]
  pub fn add(&mut self, x: &[f32]) -> Result<()> {
    unsafe {
      faiss_Index_set_verbose(self.inner, 1 as c_int);
      let n = x.len() / self.d() as usize;
      faiss_try(faiss_Index_add(self.inner, n as i64, x.as_ptr()))?;
    }
    Ok(())
  }

  #[inline]
  pub fn add_with_ids(&mut self, x: &[f32], xids: &[VectorId]) -> Result<()> {
    unsafe {
      faiss_Index_set_verbose(self.inner, 1 as c_int);
      let n = x.len() / self.d() as usize;
      faiss_try(faiss_Index_add_with_ids(
        self.inner,
        n as i64,
        x.as_ptr(),
        xids.as_ptr() as *const _,
      ))?;
    }
    Ok(())
  }

  #[inline]
  pub fn search(&self, query: &[f32], k: usize) -> Result<SearchResult> {
    unsafe {
      let d = self.d();
      let nq = query.len() / d as usize;
      let mut distances = vec![0_f32; k * nq];
      let mut labels = vec![VectorId::none(); k * nq];
      faiss_try(faiss_Index_search(
        self.inner,
        nq as faiss_idx_t,
        query.as_ptr(),
        k as faiss_idx_t,
        distances.as_mut_ptr(),
        labels.as_mut_ptr() as *mut _,
      ))?;
      Ok(SearchResult { distances, labels })
    }
  }

  #[inline]
  pub fn search_with_parameter(&self) {
    unimplemented!()
  }

  #[inline]
  pub(crate) fn read_from_file(path: &Path) -> Result<Self> {
    unsafe {
      let flags = CString::new("r").unwrap();
      let mut file = libc::fopen(
        path
          .canonicalize()
          .map_err(|e| Error::IOError(e.to_string()))?
          .to_str()
          .unwrap()
          .as_ptr() as *const i8,
        flags.as_ptr() as *const i8,
      );
      if file.is_null() {
        return Err(Error::IOError(format!(
          "Failed to open file: {}",
          path.display()
        )));
      }
      let mut index_ptr = ::std::ptr::null_mut();
      let _ = faiss_try(faiss_read_index(
        file as *mut faiss_ffi::FILE,
        libc::O_RDONLY,
        &mut index_ptr,
      ));
      Ok(Self {
        inner: index_ptr,
        params: ParameterSpace::new()?,
      })
    }
  }

  #[inline]
  pub(crate) fn write_to_file(&self, path: &Path) -> Result<()> {
    unsafe {
      let path_c = CString::new(path.to_str().unwrap()).unwrap();
      let flags = CString::new("w").unwrap();
      let mut file =
        libc::fopen(path_c.as_ptr() as *const i8, flags.as_ptr() as *const i8);
      if file.is_null() {
        return Err(Error::IOError(format!(
          "Failed to open file for writing: {:?}",
          path.display()
        )));
      }
      let _ =
        faiss_try(faiss_write_index(self.inner, file as *mut faiss_ffi::FILE));
      faiss_try(libc::fflush(file))?;
      let fd = libc::fileno(file);
      faiss_try(libc::ferror(file))?;
      faiss_try(libc::close(fd))?;
      Ok(())
    }
  }

  #[inline]
  pub(crate) fn d(&self) -> u32 {
    unsafe { faiss_Index_d(self.inner) as u32 }
  }
}
