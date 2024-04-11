use std::collections::HashMap;
use std::ffi::CStr;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

#[repr(C)]
#[derive(Debug)]
pub struct PermissionChecker {
  enabled: bool,
  cwd: PathBuf,
  allow_read: Vec<String>,
  allow_write: Vec<String>,
  finalized: bool,
}

const FS_RO: i8 = 0;
const FS_RW: i8 = 1;

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
    finalized: true,
  })
}

#[no_mangle]
pub extern "C" fn portal_create_new_permission_checker(
) -> Box<PermissionChecker> {
  let checker = Box::new(PermissionChecker {
    enabled: false,
    cwd: PathBuf::new(),
    allow_read: vec![],
    allow_write: vec![],
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
    FS_RO => {
      checker.allow_read.push(path.to_owned());
    }
    FS_RW => {
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
    FS_RO => &checker.allow_read,
    FS_RW => &checker.allow_write,
    _ => return 0,
  };
  if allowed_paths
    .iter()
    .any(|allowed_path| path.starts_with(allowed_path))
  {
    1
  } else {
    0
  }
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
