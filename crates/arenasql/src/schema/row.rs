use super::SerializedCell;

#[derive(Debug, Clone)]
pub struct RowId(u64);

impl Default for RowId {
  fn default() -> Self {
    Self(1)
  }
}

impl RowId {
  pub fn add(mut self, value: u64) -> Self {
    self.0 += value;
    self
  }

  pub fn serialize<'a>(&self) -> Vec<u8> {
    self.0.to_be_bytes().to_vec()
  }

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
