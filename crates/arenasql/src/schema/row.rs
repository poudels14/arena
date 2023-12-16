use super::SerializedCell;

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

pub trait RowTrait<'a> {
  fn project(&'a self, columns: &[usize]) -> Vec<&'a SerializedCell<'a>>;
}

impl<'a> RowTrait<'a> for Vec<SerializedCell<'a>> {
  fn project(&'a self, columns: &[usize]) -> Vec<&'a SerializedCell<'a>> {
    columns.iter().map(|col| &self[*col]).collect()
  }
}
