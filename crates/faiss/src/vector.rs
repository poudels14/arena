use crate::faiss_ffi::faiss_idx_t;

#[repr(transparent)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct VectorId(faiss_idx_t);

// Credit: faiss_rs
impl VectorId {
  /// # Panic
  ///
  /// Panics if the ID is too large (`>= 2^63`)
  #[inline]
  pub fn new(idx: u64) -> Self {
    assert!(
      idx < 0x8000_0000_0000_0000,
      "too large index value provided to Idx::new"
    );
    let idx = idx as faiss_idx_t;
    Self(idx)
  }

  #[inline]
  pub fn none() -> Self {
    Self(-1)
  }

  #[inline]
  pub fn is_none(self) -> bool {
    self.0 == -1
  }

  #[inline]
  pub fn is_some(self) -> bool {
    self.0 != -1
  }

  #[inline]
  pub fn get(self) -> Option<u64> {
    match self.0 {
      -1 => None,
      x => Some(x as u64),
    }
  }

  #[inline]
  pub fn to_native(self) -> faiss_idx_t {
    self.0
  }
}
