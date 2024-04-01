fn main() {
  prost_build::compile_protos(&["src/schema/schema.proto"], &["src/schema/"])
    .unwrap();
}
