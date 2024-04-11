use std::collections::HashMap;
use std::ffi::CStr;

#[repr(C)]
#[derive(Debug)]
pub struct PermissionChecker {
  enabled: bool,
  allow_read: Vec<String>,
  allow_write: Vec<String>,
  finalized: bool,
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_from_cli_args(
) -> Box<PermissionChecker> {
  let env_vars: HashMap<String, String> = std::env::vars().collect();
  let enabled: bool = env_vars
    .get("PORTAL_PERMISSION_ENABLED")
    .unwrap_or(&"false".to_owned())
    .parse()
    .expect("parsing PORTAL_PERMISSION_ENABLED env");

  let allow_read: Vec<String> = env_vars
    .get("PORTAL_ALLOW_READ")
    .map(|env| env.split("/").map(|s| s.to_owned()).collect())
    .unwrap_or_default();

  let allow_write: Vec<String> = env_vars
    .get("PORTAL_ALLOW_WRITE")
    .map(|env| env.split("/").map(|s| s.to_owned()).collect())
    .unwrap_or_default();

  Box::new(PermissionChecker {
    enabled,
    allow_read,
    allow_write,
    finalized: true,
  })
}

#[no_mangle]
pub extern "C" fn portal_create_new_permission_checker(
) -> Box<PermissionChecker> {
  let checker = Box::new(PermissionChecker {
    enabled: false,
    allow_read: vec![],
    allow_write: vec![],
    finalized: false,
  });

  checker
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_add_fs_permission(
  checker: *mut PermissionChecker,
  // 1 - read only
  // 2 - read/write
  permission: i8,
  path: *const i8,
) {
  let checker =
    unsafe { checker.as_mut().expect("Invalid Portal PermissionChecker") };
  if checker.finalized {
    panic!("Can't update Portal PermissionChecker after it's finalized");
  }
  let path = c_str_to_str(path);
  match permission {
    1 => {
      checker.allow_read.push(path.to_owned());
    }
    2 => {
      checker.allow_write.push(path.to_owned());
    }
    _ => {
      panic!("Invalid permission: {:?}", permission);
    }
  }
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_enable(
  checker: *mut PermissionChecker,
) {
  let checker =
    unsafe { checker.as_mut().expect("Invalid Permission Checker") };
  checker.enabled = true;
}

// After this, permissions can't be changed
#[no_mangle]
pub extern "C" fn portal_permission_checker_finalize(
  checker: *mut PermissionChecker,
) {
  let checker =
    unsafe { checker.as_mut().expect("Invalid Permission Checker") };
  checker.finalized = true;
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_pretty_print(
  checker: *mut PermissionChecker,
) {
  let checker =
    unsafe { checker.as_mut().expect("Invalid Permission Checker") };
  println!("{:#?}", checker)
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_has_fs_permission(
  checker: *const PermissionChecker,
  path: *const i8,
) -> i8 {
  let checker =
    unsafe { checker.as_ref().expect("Invalid Permission Checker") };
  if !checker.enabled {
    return 1;
  }
  let path = c_str_to_str(path);
  println!("CHECKING FS PERMISSION: {:?}", path);
  println!("checker = {:?}", checker);

  0
}

fn c_str_to_str<'a>(path: *const i8) -> &'a str {
  let c_str: &CStr = unsafe { CStr::from_ptr(path) };
  let path: &str = c_str.to_str().expect("Invalid C string");
  path
}
