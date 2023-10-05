pub trait ToBeBytes {
  fn to_be_bytes(&self) -> Vec<u8>;
}

impl ToBeBytes for (u32, &str) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [&self.0.to_be_bytes(), self.1.as_bytes()].concat()
  }
}

impl ToBeBytes for (u32, &String) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [&self.0.to_be_bytes(), self.1.as_bytes()].concat()
  }
}

impl ToBeBytes for (u32, u32) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [self.0.to_be_bytes(), self.1.to_be_bytes()].concat()
  }
}

impl ToBeBytes for (u32, u32, u32) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      self.0.to_be_bytes(),
      self.1.to_be_bytes(),
      self.2.to_be_bytes(),
    ]
    .concat()
  }
}

impl ToBeBytes for (u32, u32, &str, &str) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      &self.0.to_be_bytes(),
      &self.1.to_be_bytes(),
      self.2.as_bytes(),
      &self.3.as_bytes(),
    ]
    .concat()
  }
}

impl ToBeBytes for (u32, u32, &str, &String) {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      &self.0.to_be_bytes(),
      &self.1.to_be_bytes(),
      self.2.as_bytes(),
      &self.3.as_bytes(),
    ]
    .concat()
  }
}
