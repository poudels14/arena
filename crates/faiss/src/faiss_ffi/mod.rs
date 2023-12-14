#[allow(
  dead_code,
  non_upper_case_globals,
  non_snake_case,
  non_camel_case_types
)]
mod bindings;

use std::ffi::{c_int, CStr};

#[allow(dead_code)]
pub use bindings::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NativeError {
  /// The error code retrieved from the C API
  code: c_int,
  /// The exception's message
  msg: String,
}

impl std::fmt::Display for NativeError {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Error")
  }
}

impl std::error::Error for NativeError {}

impl NativeError {
  pub(crate) fn from_last_error(code: c_int) -> Self {
    unsafe {
      let e: *const _ = faiss_get_last_error();
      assert!(!e.is_null());
      let cstr = CStr::from_ptr(e);
      let msg: String = cstr.to_string_lossy().into_owned();
      NativeError { code, msg }
    }
  }
}

pub(crate) fn faiss_try(code: c_int) -> Result<c_int, NativeError> {
  if code != 0 {
    Err(NativeError::from_last_error(code))
  } else {
    Ok(code)
  }
}
