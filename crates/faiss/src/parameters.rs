use crate::error::Result;
use crate::faiss_ffi::{self, faiss_try};

#[derive(Debug)]
pub struct ParameterSpace {
  pub(crate) inner: *mut faiss_ffi::FaissParameterSpace,
}

#[allow(unused)]
impl ParameterSpace {
  pub(crate) fn new() -> Result<Self> {
    unsafe {
      let mut param_ptr = ::std::ptr::null_mut();
      faiss_try(faiss_ffi::faiss_ParameterSpace_new(&mut param_ptr))?;
      Ok(ParameterSpace { inner: param_ptr })
    }
  }

  pub fn display(&self) {
    unsafe { faiss_ffi::faiss_ParameterSpace_display(self.inner) }
  }
}

#[cfg(test)]
mod tests {
  use super::ParameterSpace;

  #[test]
  fn test() {
    let mut params = ParameterSpace::new().unwrap();
    params.display();
  }
}
