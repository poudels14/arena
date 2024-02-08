use std::path::{Path, PathBuf};
use std::{env, fs};

use tar::{Builder, Header};

fn main() {
  create_pyodide_archive();
}

fn create_pyodide_archive() {
  let files = vec![
    "pyodide-lock.json",
    "pyodide.asm.wasm",
    "python_stdlib.zip",
    "numpy-1.26.1-cp312-cp312-emscripten_3_1_52_wasm32.whl",
    "matplotlib-3.5.2-cp312-cp312-emscripten_3_1_52_wasm32.whl",
    "cycler-0.11.0-py3-none-any.whl",
    "six-1.16.0-py2.py3-none-any.whl",
    "fonttools-4.42.1-py3-none-any.whl",
    "kiwisolver-1.4.4-cp312-cp312-emscripten_3_1_52_wasm32.whl",
    "packaging-23.2-py3-none-any.whl",
    "Pillow-10.0.0-cp312-cp312-emscripten_3_1_52_wasm32.whl",
    "pyparsing-3.1.1-py3-none-any.whl",
    "python_dateutil-2.8.2-py2.py3-none-any.whl",
    "pytz-2023.3-py2.py3-none-any.whl",
    "matplotlib_pyodide-0.2.0-py3-none-any.whl",
    "portal-0.0.1-py3-none-any.whl",
  ];

  let root = Path::new("../../pyodide/dist/");
  let mut archive = Builder::new(Vec::new());
  files.iter().for_each(|file| {
    let filepath = root.join(file);
    println!(
      "cargo:rerun-if-changed={}",
      filepath.clone().to_str().unwrap()
    );
    let content = fs::read(&filepath).unwrap();
    let mut header = Header::new_gnu();
    header.set_path(file).unwrap();
    header.set_size(content.len() as u64);
    header.set_cksum();

    archive
      .append_data(&mut header, file, &mut fs::File::open(filepath).unwrap())
      .unwrap();
  });

  let archive_content = archive.into_inner().unwrap();
  let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

  fs::write(out_dir.join("pyodide.tar.gz"), &archive_content).unwrap();
}
