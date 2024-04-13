use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use libc::c_int;
use parking_lot::RwLock;

#[repr(C)]
#[derive(Debug)]
pub struct PermissionChecker {
  enabled: bool,
  cwd: PathBuf,
  allow_read: Vec<String>,
  allow_write: Vec<String>,
  track_write: bool,
  state: Arc<RwLock<State>>,
  finalized: bool,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct State {
  // list of paths opened with write access
  paths_written: Vec<String>,
}

const FS_R: i8 = 0;
const FS_W: i8 = 1;

#[no_mangle]
pub extern "C" fn portal_permission_checker_from_cli_args(
) -> Box<PermissionChecker> {
  let env_vars: HashMap<String, String> = std::env::vars().collect();
  let enabled: bool = env_vars
    .get("PORTAL_PERMISSION_ENABLED")
    .unwrap_or(&"false".to_owned())
    .parse()
    .unwrap_or_default();

  let allow_read: Vec<String> = env_vars
    .get("PORTAL_ALLOW_READ")
    .map(|env| {
      env
        .split(":")
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
          let path = normalize_path(s);
          if !path.is_absolute() {
            None
          } else {
            Some(
              path
                .as_os_str()
                .to_str()
                .expect("converting path to string")
                .to_owned(),
            )
          }
        })
        .collect()
    })
    .unwrap_or_default();

  let track_write: bool = env_vars
    .get("PORTAL_TRACK_WRITE")
    .unwrap_or(&"false".to_owned())
    .parse()
    .unwrap_or_default();

  let allow_write: Vec<String> = env_vars
    .get("PORTAL_ALLOW_WRITE")
    .map(|env| {
      env
        .split(":")
        .filter(|s| !s.is_empty())
        .filter_map(|s| {
          let path = normalize_path(s);
          if !path.is_absolute() {
            None
          } else {
            Some(
              path
                .as_os_str()
                .to_str()
                .expect("converting path to string")
                .to_owned(),
            )
          }
        })
        .collect()
    })
    .unwrap_or_default();

  let cwd = std::env::current_dir().expect("getting current dir");
  Box::new(PermissionChecker {
    enabled,
    cwd,
    allow_read,
    allow_write,
    track_write,
    state: Arc::new(RwLock::new(State::default())),
    finalized: true,
  })
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_create_new(
) -> Box<PermissionChecker> {
  let checker = Box::new(PermissionChecker {
    enabled: false,
    cwd: PathBuf::new(),
    allow_read: vec![],
    allow_write: vec![],
    track_write: false,
    state: Arc::new(RwLock::new(State::default())),
    finalized: false,
  });

  checker
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_add_fs_permission(
  checker: *mut PermissionChecker,
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
    FS_R => {
      checker.allow_read.push(path.to_owned());
    }
    FS_W => {
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
  permission: i8,
  path: *const i8,
) -> i8 {
  let checker =
    unsafe { checker.as_ref().expect("Invalid Permission Checker") };
  if !checker.enabled {
    return 1;
  }
  let mut path = c_str_to_str(path);
  #[allow(unused)]
  let mut joined_path = PathBuf::new();
  if path.starts_with(".") {
    joined_path = checker.cwd.join(path);
    path = joined_path.to_str().expect("normalizing path");
  }
  log::debug!(
    "Checking FS permission[{:?}] for path: {:?}",
    permission,
    path
  );

  let allowed_paths = match permission {
    FS_R => &checker.allow_read,
    FS_W => &checker.allow_write,
    _ => return 0,
  };
  if allowed_paths
    .iter()
    .any(|allowed_path| path.starts_with(allowed_path))
  {
    if permission == FS_W && checker.track_write {
      checker.state.write().paths_written.push(path.to_owned());
    }
    1
  } else {
    0
  }
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_list_paths_written(
  checker: *const PermissionChecker,
  outlen: *mut c_int,
) -> *mut *mut i8 {
  let checker =
    unsafe { checker.as_ref().expect("Invalid Permission Checker") };
  let state_paths_written = &checker.state.read().paths_written;

  let mut out = state_paths_written
    .iter()
    .map(|path| CString::new(path.as_str()).unwrap().into_raw())
    .collect::<Vec<*mut i8>>();

  out.shrink_to_fit();
  let len = out.len();
  let ptr = out.as_mut_ptr();
  std::mem::forget(out);
  unsafe {
    std::ptr::write(outlen, len as c_int);
  }

  ptr
}

#[no_mangle]
pub extern "C" fn portal_permission_checker_reset_paths_written(
  checker: *const PermissionChecker,
) {
  let checker =
    unsafe { checker.as_ref().expect("Invalid Permission Checker") };
  checker.state.write().paths_written.clear();
}

fn c_str_to_str<'a>(path: *const i8) -> &'a str {
  let c_str: &CStr = unsafe { CStr::from_ptr(path) };
  let path: &str = c_str.to_str().expect("Invalid C string");
  path
}

// Credit: deno
/// Normalize all intermediate components of the path (ie. remove "./" and "../" components).
/// Similar to `fs::canonicalize()` but doesn't resolve symlinks.
///
/// Taken from Cargo
/// <https://github.com/rust-lang/cargo/blob/af307a38c20a753ec60f0ad18be5abed3db3c9ac/src/cargo/util/paths.rs#L60-L85>
#[inline]
pub fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
  let mut components = path.as_ref().components().peekable();
  let mut ret =
    if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
      components.next();
      PathBuf::from(c.as_os_str())
    } else {
      PathBuf::new()
    };

  for component in components {
    match component {
      Component::Prefix(..) => unreachable!(),
      Component::RootDir => {
        ret.push(component.as_os_str());
      }
      Component::CurDir => {}
      Component::ParentDir => {
        ret.pop();
      }
      Component::Normal(c) => {
        ret.push(c);
      }
    }
  }
  ret
}
