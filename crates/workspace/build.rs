use glob::glob;
use std::env;
use std::fs::File;
use std::path::PathBuf;
use tar::Builder;

pub fn main() {
  let o = PathBuf::from(env::var_os("OUT_DIR").unwrap());

  let file = File::create(o.join("DEFAULT_WORKSPACE_TEMPLATE.tar")).unwrap();
  let mut a = Builder::new(file);

  let mut paths = Vec::new();

  glob("./template/arena.config.toml")
    .unwrap()
    .for_each(|p| paths.push(p));
  glob("./template/tsconfig.json")
    .unwrap()
    .for_each(|p| paths.push(p));
  glob("./template/entry-server.tsx")
    .unwrap()
    .for_each(|p| paths.push(p));
  glob("./template/.gitignore")
    .unwrap()
    .for_each(|p| paths.push(p));
  // Copy all files under template/root in new workspace
  glob("./template/src/**/*")
    .unwrap()
    .for_each(|p| paths.push(p));

  for entry in paths {
    match entry {
      Ok(path) => {
        a.append_path_with_name(
          path.clone(),
          path
            .clone()
            .strip_prefix("template")
            .expect("failed to strip template prefix"),
        )
        .unwrap();
      }
      Err(e) => panic!("ERROR: {:?}", e),
    }
  }
}
