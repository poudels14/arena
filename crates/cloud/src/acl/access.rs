use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Access {
  View,
  Edit,
  Admin,
  Owner,
  Unknown,
}

impl Access {
  pub fn from(value: &str) -> Access {
    match value {
      "view" => Access::View,
      "edit" => Access::Edit,
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
      Self::Edit if other == &Self::Admin || other == &Self::Owner => -1,
      Self::Edit => 1,
      _ => -1,
    }
  }
}

mod tests {
  #[allow(unused_imports)]
  use crate::acl::Access::{Admin, Edit, Owner, Unknown, View};

  #[test]
  fn test_compare_owner() {
    assert_eq!(Owner.compare(&Owner), 0);
    assert_eq!(Owner.compare(&Admin), 1);
    assert_eq!(Owner.compare(&Edit), 1);
    assert_eq!(Owner.compare(&View), 1);
    assert_eq!(Owner.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_admin() {
    assert_eq!(Admin.compare(&Owner), -1);
    assert_eq!(Admin.compare(&Admin), 0);
    assert_eq!(Admin.compare(&Edit), 1);
    assert_eq!(Admin.compare(&View), 1);
    assert_eq!(Admin.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_can_mutate() {
    assert_eq!(Edit.compare(&Owner), -1);
    assert_eq!(Edit.compare(&Admin), -1);
    assert_eq!(Edit.compare(&Edit), 0);
    assert_eq!(Edit.compare(&View), 1);
    assert_eq!(Edit.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_can_query() {
    assert_eq!(View.compare(&Owner), -1);
    assert_eq!(View.compare(&Admin), -1);
    assert_eq!(View.compare(&Edit), -1);
    assert_eq!(View.compare(&View), 0);
    assert_eq!(View.compare(&Unknown), -1);
  }

  #[test]
  fn test_compare_unknown() {
    assert_eq!(Unknown.compare(&Owner), -1);
    assert_eq!(Unknown.compare(&Admin), -1);
    assert_eq!(Unknown.compare(&Edit), -1);
    assert_eq!(Unknown.compare(&View), -1);
    assert_eq!(Unknown.compare(&Unknown), 0);
  }
}
