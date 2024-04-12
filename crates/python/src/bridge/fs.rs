use std::ffi::CString;

#[no_mangle]
pub extern "C" fn portal_bridge_fs_get_artifacts_path() -> *const i8 {
  let home = dirs::home_dir();
  let home = home.map(|h| {
    let data_path = h.join("Desktop").join("PortalAI");

    if !data_path.exists() {
      std::fs::create_dir_all(&data_path)
        .expect(&format!("Creating PortalAI dir: {:?}", &data_path))
    }
    data_path
  });

  let path =
    home.and_then(|home| home.to_str().and_then(|s| CString::new(s).ok()));
  match path {
    Some(s) => return s.into_raw(),
    None => std::ptr::null(),
  }
}
