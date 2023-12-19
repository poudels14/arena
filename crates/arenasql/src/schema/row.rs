use super::{OwnedSerializedCell, SerializedCell};
use crate::storage::Serializer;

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
    Self::serialize_u64(self.0)
  }

  #[inline]
  pub fn serialize_u64<'a>(value: u64) -> Vec<u8> {
    Serializer::VarInt.serialize(&value).unwrap()
  }

  #[inline]
  pub fn deserialize(bytes: &[u8]) -> RowId {
    RowId(
      Serializer::VarInt
        .deserialize(bytes.try_into().unwrap())
        .unwrap(),
    )
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

#[cfg(test)]
mod tests {
  use crate::storage::Serializer;

  #[test]
  /// This checks the serialized row id bytes comparision to make sure
  /// that sorting by serialized bytes is same as sorting by its u64
  /// value. That needs to hold true so that rocksdb can be scanned
  /// accurately in ASC/DESC order
  fn test_row_id_serialized_sort() {
    let mut prev = Serializer::VarInt.serialize::<u64>(&0).unwrap();
    // Checked this upto 10 billion manually already
    for i in 1..1_000_000 {
      let next = Serializer::VarInt.serialize::<u64>(&i).unwrap();
      if prev >= next {
        panic!("Serialized row ordering failed at index: {}", i);
      }
      prev = next;
    }
  }
}
