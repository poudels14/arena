use std::env;

pub fn main() {
  println!("cargo:rustc-env=TARGET={}", env::var("TARGET").unwrap());
}
