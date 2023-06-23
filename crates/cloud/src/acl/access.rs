use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Access {
  CanQuery,
  CanMutate,
  Admin,
  Owner,
  Unknown,
}

impl Access {
  pub fn from(value: &str) -> Access {
    match value {
      "can-query" => Access::CanQuery,
      "can-mutate" => Access::CanMutate,
      "admin" => Access::Admin,
      "owner" => Access::Owner,
      _ => Access::Unknown,
    }
  }

  pub fn compare(&self, other: &Access) -> i16 {
    if self == other {
      return 0;
    }

    // If unknown access is being compared, return -1
    if self == &Self::Unknown || other == &Self::Unknown {
      return -1;
    }

    match self {
      Self::Owner => 1,
      Self::Admin if other == &Self::Owner => -1,
      Self::Admin => 1,
      Self::CanMutate if other == &Self::Admin || other == &Self::Owner => -1,
      Self::CanMutate => 1,
      _ => -1,
    }
  }
}

mod tests {
  #[allow(unused_imports)]
  use crate::acl::Access::{Admin, CanMutate, CanQuery, Owner, Unknown};

  #[test]
  fn test_compare_owner() {
    assert_eq!(Owner.compare(&Owner), 0);
    assert_eq!(Owner.compare(&Admin), 1);
    assert_eq!(Owner.compare(&CanMutate), 1);
    assert_eq!(Owner.compare(&CanQuery), 1);
    assert_eq!(Owner.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_admin() {
    assert_eq!(Admin.compare(&Owner), -1);
    assert_eq!(Admin.compare(&Admin), 0);
    assert_eq!(Admin.compare(&CanMutate), 1);
    assert_eq!(Admin.compare(&CanQuery), 1);
    assert_eq!(Admin.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_can_mutate() {
    assert_eq!(CanMutate.compare(&Owner), -1);
    assert_eq!(CanMutate.compare(&Admin), -1);
    assert_eq!(CanMutate.compare(&CanMutate), 0);
    assert_eq!(CanMutate.compare(&CanQuery), 1);
    assert_eq!(CanMutate.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_can_query() {
    assert_eq!(CanQuery.compare(&Owner), -1);
    assert_eq!(CanQuery.compare(&Admin), -1);
    assert_eq!(CanQuery.compare(&CanMutate), -1);
    assert_eq!(CanQuery.compare(&CanQuery), 0);
    assert_eq!(CanQuery.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_unknown() {
    assert_eq!(Unknown.compare(&Owner), -1);
    assert_eq!(Unknown.compare(&Admin), -1);
    assert_eq!(Unknown.compare(&CanMutate), -1);
    assert_eq!(Unknown.compare(&CanQuery), -1);
    assert_eq!(Unknown.compare(&Unknown), 0);
  }
}
