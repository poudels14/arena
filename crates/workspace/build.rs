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

  vec![
    "../../js/templates/default/arena.config.toml",
    "../../js/templates/default/tsconfig.json",
    "../../js/templates/default/entry-server.tsx",
    "../../js/templates/default/entry-client.tsx",
    "../../js/templates/default/.gitignore",
    // Copy all files under template/root in new workspace
    "../../js/templates/default/src/**/*",
  ]
  .iter()
  .for_each(|pattern| {
    glob(pattern).unwrap().for_each(|p| paths.push(p));
  });

  for entry in paths {
    match entry {
      Ok(path) => {
        a.append_path_with_name(
          path.clone(),
          path
            .clone()
            .strip_prefix("../../js/templates/default")
            .expect("failed to strip template prefix"),
        )
        .unwrap();
      }
      Err(e) => panic!("ERROR: {:?}", e),
    }
  }
}
