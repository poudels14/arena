use std::ffi::CString;

use libc::c_int;

pub mod fs;
pub mod permission;

#[no_mangle]
pub unsafe extern "C" fn portal_bridge_free_string(ptr: *const char) {
  let _ = CString::from_raw(ptr as *mut _);
}

#[no_mangle]
unsafe extern "C" fn portal_bridge_free_string_array(
  ptr: *mut *mut i8,
  len: c_int,
) {
  let len = len as usize;
  let arr = Vec::from_raw_parts(ptr, len, len);
  for ele in arr {
    let s = CString::from_raw(ele);
    std::mem::drop(s);
  }
}
