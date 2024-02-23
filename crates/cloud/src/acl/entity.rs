use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AclEntity {
  App {
    id: String,
    path: Option<String>,
    entity: Option<Value>,
  },
  Resource(String),
  Unknown,
}

impl AclEntity {
  pub fn get_id<'a>(&'a self) -> &'a str {
    match self {
      Self::App { id, .. } => id,
      Self::Resource(id) => id,
      _ => panic!("Can't get id of unknown resource"),
    }
  }

  pub fn comapre(&self, other: &Self) -> i16 {
    if self == other {
      return 0;
    }
    match self {
      Self::App { id, path, .. } => match other {
        Self::App { id: other_id, .. } => {
          if (id == "*" || id == other_id) && path.is_none() {
            return 1;
          }
          return -1;
        }
        _ => -1,
      },
      Self::Resource(id) => match other {
        Self::Resource(_) if id == "*" => 1,
        _ => -1,
      },
      _ => -1,
    }
  }
}

#[allow(unused)]
mod tests {
  use super::AclEntity;
  use crate::acl::AclEntity::{App, Resource};

  #[test]
  fn test_compare_app() {
    assert_eq!(app("*", None).comapre(&app("*", None)), 0);

    assert_eq!(app("*", None).comapre(&app("1", None)), 1);
    assert_eq!(app("1", None).comapre(&app("1", Some("/p"))), 1);
    assert_eq!(app("1", None).comapre(&app("1", None)), 0);
    assert_eq!(app("1", None).comapre(&app("*", None)), -1);

    assert_eq!(app("1", Some("/p")).comapre(&app("1", Some("/p"))), 0);
    assert_eq!(app("1", Some("/p")).comapre(&app("1", Some("/p2"))), -1);
    assert_eq!(app("1", Some("/p")).comapre(&app("1", None)), -1);
    assert_eq!(app("1", Some("/p")).comapre(&app("*", Some("/p"))), -1);

    assert_eq!(app("1", None).comapre(&app("2", None)), -1);
  }

  #[test]
  fn test_compare_resource() {
    assert_eq!(resource("*").comapre(&resource("*")), 0);

    assert_eq!(resource("*").comapre(&resource("1")), 1);
    assert_eq!(resource("1").comapre(&resource("1")), 0);
    assert_eq!(resource("1").comapre(&resource("*")), -1);
    assert_eq!(resource("1").comapre(&resource("2")), -1);
  }

  #[test]
  fn test_compare_app_and_resource() {
    assert_eq!(app("*", None).comapre(&resource("*")), -1);
    assert_eq!(resource("*").comapre(&app("*", None)), -1);
  }

  fn app(id: &str, path: Option<&str>) -> AclEntity {
    App {
      id: id.to_owned(),
      path: path.map(|p| p.to_owned()),
      entity: None,
    }
  }

  fn resource(id: &str) -> AclEntity {
    Resource(id.to_owned())
  }
}
