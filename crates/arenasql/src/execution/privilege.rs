use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{ObjectType, Statement as SQLStatement};

bitflags! {
  #[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
  )]
  pub struct Privilege: u64 {
    const SUPER_USER = u64::MAX;
    const READ_ONLY = Self::READ_TABLE_SCHEMA.bits() | Self::SELECT_ROWS.bits();
    // Database level privileges
    const CREATE_DATABASE = 1 << 47;
    // Privileges for system schema like pg_user/pg_database, etc
    const ALTER_SYSTEM_SCHEMA = 1 << 46;
    const READ_SYSTEM_SCHEMA = 1 << 45;
    // Table level privileges
    const CREATE_TABLE = 1 << 31;
    const DROP_TABLE = 1 << 30;
    const ALTER_TABLE = 1 << 29;
    const READ_TABLE_SCHEMA = 1 << 28;
    // Table privileges will allow all row level privileges too
    const TABLE_PRIVILEGES = Self::CREATE_TABLE.bits()
      | Self::DROP_TABLE.bits()
      | Self::ALTER_TABLE.bits()
      | Self::READ_TABLE_SCHEMA.bits()
      | Self::ROWS_PRIVILEGES.bits();
    // Rows level privileges
    const SELECT_ROWS = 1 << 12;
    const INSERT_ROWS = 1 << 13;
    const UPDATE_ROWS = 1 << 14 | Self::SELECT_ROWS.bits();
    const DELETE_ROWS = 1 << 15 | Self::SELECT_ROWS.bits();
    const ROWS_PRIVILEGES = Self::SELECT_ROWS.bits()
      | Self::INSERT_ROWS.bits()
      | Self::UPDATE_ROWS.bits()
      | Self::DELETE_ROWS.bits();

    const SET_SESSION_PARAMS = 1 << 6;
    // No privilege - for default
    const NONE = 0;
  }
}

impl Default for Privilege {
  fn default() -> Self {
    Self::NONE
  }
}

impl Privilege {
  #[inline]
  pub fn get_required_privilege(stmt: &SQLStatement) -> Self {
    match stmt {
      // Database
      SQLStatement::CreateDatabase { .. } => Self::CREATE_DATABASE,
      // Table
      SQLStatement::CreateTable { .. } | SQLStatement::CreateIndex { .. } => {
        Self::CREATE_TABLE
      }
      SQLStatement::AlterTable { .. } | SQLStatement::AlterIndex { .. } => {
        Self::ALTER_TABLE
      }
      SQLStatement::Explain { .. } => Self::READ_TABLE_SCHEMA,
      // Rows
      SQLStatement::Insert { .. } => Self::INSERT_ROWS,
      SQLStatement::Delete { .. } => Self::DELETE_ROWS,
      SQLStatement::Update { .. } => Self::UPDATE_ROWS,
      SQLStatement::Query(_) => Self::SELECT_ROWS,
      // Statements done need privilege
      SQLStatement::StartTransaction { .. }
      | SQLStatement::Commit { .. }
      | SQLStatement::Rollback { .. } => Self::NONE,
      // Drop
      SQLStatement::Drop { object_type, .. } => match object_type {
        ObjectType::Table | ObjectType::Index => Self::DROP_TABLE,
        _ => Self::SUPER_USER,
      },
      SQLStatement::SetTimeZone { .. } => Self::SET_SESSION_PARAMS,
      SQLStatement::SetVariable { .. } => Self::SET_SESSION_PARAMS,
      // For rest of the statements, require super user privilege
      _ => Self::SUPER_USER,
    }
  }

  pub fn can_execute(&self, stmt: &SQLStatement) -> bool {
    let required = Self::get_required_privilege(stmt);
    (*self & required) == required
  }
}

#[cfg(test)]
mod tests {
  use crate::ast::parse;
  use crate::execution::Privilege;
  use sqlparser::ast::{Ident, ObjectName, Statement as SQLStatement};

  #[test]
  fn privilege_test_super_user_privileges() {
    assert!(
      (Privilege::SUPER_USER.bits() & Privilege::SELECT_ROWS.bits()) > 0,
      "Expected SUPER_USER to have SELECT_ROWS privilege"
    );

    assert!(
      (Privilege::SUPER_USER.bits() & Privilege::CREATE_DATABASE.bits()) > 0,
      "Expected SUPER_USER top have CREATE_DATABASE privilege"
    );
  }

  #[test]
  fn privilege_test_database_level_privileges() {
    assert_eq!(
      Privilege::CREATE_DATABASE.bits(),
      1 + u64::MAX / (2 << 16),
      "CREATE_DATABASE flag didn't match"
    );
  }

  #[test]
  fn privilege_test_schema_level_privileges() {
    assert_eq!(
      Privilege::ALTER_SYSTEM_SCHEMA.bits(),
      1 + u64::MAX / (2 << 16) / 2,
      "ALTER_SYSTEM_SCHEMA flag didn't match"
    );

    assert_eq!(
      Privilege::READ_SYSTEM_SCHEMA.bits(),
      1 + u64::MAX / (2 << 16) / 4,
      "READ_SYSTEM_SCHEMA flag didn't match"
    );
  }

  #[test]
  fn privilege_test_table_level_privileges() {
    assert_eq!(
      Privilege::CREATE_TABLE.bits(),
      1 + u32::MAX as u64 / 2,
      "CREATE_TABLE flag didn't match"
    );

    assert_eq!(
      Privilege::DROP_TABLE.bits(),
      1 + u32::MAX as u64 / 4,
      "DROP_TABLE flag didn't match"
    );

    assert_eq!(
      Privilege::ALTER_TABLE.bits(),
      1 + u32::MAX as u64 / 8,
      "ALTER_TABLE flag didn't match"
    );
  }

  #[test]
  fn privilege_test_rows_level_privileges() {
    assert_eq!(
      Privilege::SELECT_ROWS.bits(),
      1 + u16::MAX as u64 / 16,
      "SELECT flag didn't match"
    );
    assert_eq!(
      Privilege::INSERT_ROWS.bits(),
      1 + u16::MAX as u64 / 8,
      "INSERT flag didn't match"
    );
    assert_eq!(
      Privilege::UPDATE_ROWS.bits(),
      (1 + u16::MAX as u64 / 4) | Privilege::SELECT_ROWS.bits(),
      "UPDATE flag didn't match"
    );
    assert_eq!(
      Privilege::DELETE_ROWS.bits(),
      (1 + u16::MAX as u64 / 2) | Privilege::SELECT_ROWS.bits(),
      "DELETE flag didn't match"
    );

    assert_eq!(
      (Privilege::INSERT_ROWS
        | Privilege::DELETE_ROWS
        | Privilege::UPDATE_ROWS
        | Privilege::SELECT_ROWS)
        .bits(),
      15 << 12 as u64,
      "SELECT flag didn't match"
    );
  }

  #[test]
  fn privilege_test_can_execute() {
    assert_eq!(
      Privilege::CREATE_DATABASE.can_execute(&SQLStatement::CreateDatabase {
        db_name: ObjectName(vec![Ident::new("db")]),
        if_not_exists: false,
        location: None,
        managed_location: None
      }),
      true,
      "Expected CREATE_DATABASE privilege to execute CreateDatabase"
    );

    assert!(
      !Privilege::TABLE_PRIVILEGES.can_execute(&SQLStatement::CreateDatabase {
        db_name: ObjectName(vec![Ident::new("db")]),
        if_not_exists: false,
        location: None,
        managed_location: None
      }),
      "Expected TABLE privileges to NOT execute CreateDatabase"
    );

    assert!(
      !Privilege::SELECT_ROWS.can_execute(&SQLStatement::CreateDatabase {
        db_name: ObjectName(vec![Ident::new("db")]),
        if_not_exists: false,
        location: None,
        managed_location: None
      }),
      "Expected SELECT_ROWS privileges to NOT execute CreateDatabase"
    );

    let select_query = parse("SELECT * from users").unwrap().pop().unwrap();
    let insert_query =
      parse("INSERT INTO users VALUES(1)").unwrap().pop().unwrap();

    assert!(
      Privilege::SELECT_ROWS.can_execute(&select_query),
      "Expected SELECT_ROWS privileges to execute SELECT query"
    );

    assert!(
      !Privilege::SELECT_ROWS.can_execute(&insert_query),
      "Expected SELECT_ROWS privileges to NOT execute INSERT query"
    );

    assert!(
      Privilege::UPDATE_ROWS.can_execute(&select_query),
      "Expected UPDATE_ROWS privileges to execute SELECT query"
    );

    assert!(
      !Privilege::INSERT_ROWS.can_execute(&select_query),
      "Expected INSERT_ROWS privileges to NOT execute SELECT query"
    );
  }
}
