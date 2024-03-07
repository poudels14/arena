use serde_json::Value;
use sqlx::types::chrono::NaiveDateTime;
use sqlx::FromRow;

#[derive(FromRow, Debug, Clone)]
pub struct Acl {
  pub id: String,
  pub workspace_id: String,
  pub user_id: String,
  pub app_id: Option<String>,
  pub resource_id: Option<String>,
  // pub access: String,
  // metadata will contain following fields:
  //  - (old) table: table name; `*` for all table access
  //  - (old) filter: SQL filter; `*` for no filter
  //  - filters: {
  //        READ, WRITE, UPDATE, DELETE
  //        corresponds to: SELECT, INSERT, UPDATE, DELETE sql query
  //        * will give all access
  //      command: string,
  //      table: string,
  //      condition: string,
  //    }[]
  pub metadata: Value,
  pub archived_at: Option<NaiveDateTime>,
}
