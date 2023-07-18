use bstr::BStr;
use bstr::ByteSlice;

pub trait ToBeBytes {
  fn to_be_bytes(&self) -> Vec<u8>;
}

impl ToBeBytes for (i32, i32) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [self.0.to_be_bytes(), self.1.to_be_bytes()].concat()
  }
}

impl ToBeBytes for (i32, &BStr) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [&self.0.to_be_bytes(), self.1.as_bytes()].concat()
  }
}

impl ToBeBytes for (i32, i32, u32) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      self.0.to_be_bytes(),
      self.1.to_be_bytes(),
      self.2.to_be_bytes(),
    ]
    .concat()
  }
}
