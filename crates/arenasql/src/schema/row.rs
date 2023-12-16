use super::{OwnedSerializedCell, SerializedCell};

#[derive(Debug, Clone)]
pub struct RowId(pub u64);

impl Default for RowId {
  fn default() -> Self {
    Self(1)
  }
}

impl RowId {
  #[inline]
  pub fn value(&self) -> u64 {
    self.0
  }

  #[inline]
  pub fn add(mut self, value: u64) -> Self {
    self.0 += value;
    self
  }

  #[inline]
  pub fn serialize<'a>(&self) -> Vec<u8> {
    self.0.to_be_bytes().to_vec()
  }

  #[inline]
  pub fn serialize_u64<'a>(value: u64) -> [u8; 8] {
    value.to_be_bytes()
  }

  #[inline]
  pub fn deserialize(bytes: &[u8]) -> RowId {
    RowId(u64::from_be_bytes(bytes.try_into().unwrap()))
  }
}

pub type Row<'a> = Vec<SerializedCell<'a>>;
pub type OwnedRow = Vec<OwnedSerializedCell>;

pub trait RowTrait<'a, O> {
  fn project(&'a self, columns: &[usize]) -> Vec<&'a O>;
}

impl<'a> RowTrait<'a, SerializedCell<'a>> for Row<'a> {
  fn project(&'a self, columns: &[usize]) -> Vec<&'a SerializedCell<'a>> {
    columns.iter().map(|col| &self[*col]).collect()
  }
}

impl<'a> RowTrait<'a, OwnedSerializedCell> for OwnedRow {
  fn project(&'a self, columns: &[usize]) -> Vec<&'a OwnedSerializedCell> {
    columns.iter().map(|col| &self[*col]).collect()
  }
}
