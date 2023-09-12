use crate::utils::bytes::ToBeBytes;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RowId {
  pub collection_index: u32,
  pub row_index: u32,
}

impl ToBeBytes for RowId {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      self.collection_index.to_be_bytes(),
      self.row_index.to_be_bytes(),
    ]
    .concat()
  }
}

impl ToBeBytes for &RowId {
  fn to_be_bytes(&self) -> Vec<u8> {
    [
      self.collection_index.to_be_bytes(),
      self.row_index.to_be_bytes(),
    ]
    .concat()
  }
}
