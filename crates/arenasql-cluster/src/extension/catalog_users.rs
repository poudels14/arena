use std::pin::Pin;
use std::sync::Arc;

use arenasql::datafusion::{
  ColumnarValue, DatafusionDataType as DfDataType, DatafusionField as Field,
  ScalarValue, Schema, SchemaRef, TaskContext,
};
use arenasql::execution::tablescan::HeapIterator;
use arenasql::execution::{
  convert_literals_to_columnar_values, CustomExecutionPlan,
  ExecutionPlanResponse, Transaction,
};
use arenasql::schema::{DataFrame, DataType, SerializedCell};
use arenasql::storage::Serializer;
use arenasql::{Error, Result};
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Expr;

// Note: store user/password config as a single column since it will be
// difficult to update this table once there are databases in prod
static CREATE_USERS_TABLE: &'static str =
  "CREATE TABLE IF NOT EXISTS arena_catalog.users(config TEXT);";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CatalogUser {
  catalog: String,
  username: String,
  password: String,
}

#[derive(Clone)]
pub struct SetCatalogUserCredentials {
  transaction: Transaction,
  user: CatalogUser,
}

impl SetCatalogUserCredentials {
  pub fn new(transaction: Transaction, parameters: &Vec<Expr>) -> Result<Self> {
    let args = convert_literals_to_columnar_values(
      &vec![DfDataType::Utf8, DfDataType::Utf8, DfDataType::Utf8],
      &parameters,
    )?;

    let catalog = args
      .get(0)
      .and_then(get_scalar_string)
      .ok_or(Error::InvalidQuery(format!("Catalog missing")))?;
    let username = args
      .get(1)
      .and_then(get_scalar_string)
      .ok_or(Error::InvalidQuery(format!("Username missing")))?;
    let password = args
      .get(2)
      .and_then(get_scalar_string)
      .ok_or(Error::InvalidQuery(format!("Password missing")))?;

    Ok(Self {
      transaction,
      user: CatalogUser {
        catalog,
        username,
        password,
      },
    })
  }
}

impl CustomExecutionPlan for SetCatalogUserCredentials {
  fn schema(&self) -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
      Field::new("catalog", DfDataType::Utf8, false),
      Field::new("user", DfDataType::Utf8, false),
      Field::new("password", DfDataType::Utf8, false),
    ]))
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<DataFrame>> + Send>>> {
    let plan = self.clone();
    let query = async move {
      plan.transaction.execute_sql(CREATE_USERS_TABLE).await?;

      let mut dataframe = DataFrame::with_capacity(
        10,
        vec![
          ("catalog".to_owned(), DataType::Text),
          ("user".to_owned(), DataType::Text),
          ("password".to_owned(), DataType::Text),
        ],
      );

      let txn = plan.transaction.storage_transaction();
      let users_table =
        txn.state().get_table("arena_catalog", "users").unwrap();
      let storage = txn.lock(false)?;
      let cols = vec![0];

      let mut rows_iter = HeapIterator::new(&storage, &users_table, &cols);
      let existing_users = scan_catalog_users(&mut rows_iter, &txn.serializer)?;

      for (row_id, user) in existing_users {
        if user.catalog == plan.user.catalog
          && user.username == plan.user.username
        {
          storage.delete_row(&users_table, &row_id)?;
        }
        dataframe.append_row(
          &row_id,
          &vec![
            &SerializedCell::Blob(user.catalog.as_bytes()),
            &SerializedCell::Blob(user.username.as_bytes()),
            &SerializedCell::Blob(user.password.as_bytes()),
          ],
        );
      }

      let row_id = storage.generate_next_row_id(&users_table)?;
      storage.insert_row(
        &users_table,
        &row_id,
        &vec![SerializedCell::Blob(&txn.serializer.serialize(&plan.user)?)],
      )?;

      Ok(dataframe)
    };
    Ok(Box::pin(futures::stream::once(query)))
  }
}

#[derive(Clone)]
pub struct ListCatalogUserCredentials {
  transaction: Transaction,
  catalog: String,
}

impl ListCatalogUserCredentials {
  pub fn new(transaction: Transaction, parameters: &Vec<Expr>) -> Result<Self> {
    let args = convert_literals_to_columnar_values(
      &vec![DfDataType::Utf8],
      &parameters,
    )?;

    let catalog = args
      .get(0)
      .and_then(get_scalar_string)
      .ok_or(Error::InvalidQuery(format!("Catalog missing")))?;

    Ok(Self {
      catalog,
      transaction,
    })
  }
}

impl CustomExecutionPlan for ListCatalogUserCredentials {
  fn schema(&self) -> SchemaRef {
    SchemaRef::new(Schema::new(vec![
      Field::new("catalog", DfDataType::Utf8, false),
      Field::new("user", DfDataType::Utf8, false),
      Field::new("password", DfDataType::Utf8, false),
    ]))
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<ExecutionPlanResponse> {
    let plan = self.clone();
    let query = async move {
      let mut dataframe = DataFrame::with_capacity(
        10,
        vec![
          ("catalog".to_owned(), DataType::Text),
          ("user".to_owned(), DataType::Text),
          ("password".to_owned(), DataType::Text),
        ],
      );

      let txn = plan.transaction.storage_transaction();
      let users_table =
        txn.state().get_table("arena_catalog", "users").unwrap();
      let storage = txn.lock(false)?;
      let cols = vec![0];

      let mut rows_iter = HeapIterator::new(&storage, &users_table, &cols);
      let existing_users = scan_catalog_users(&mut rows_iter, &txn.serializer)?;

      existing_users
        .iter()
        .filter(|c| c.1.catalog == plan.catalog)
        .for_each(|(row_id, user)| {
          dataframe.append_row(
            &row_id,
            &vec![
              &SerializedCell::Blob(user.catalog.as_bytes()),
              &SerializedCell::Blob(user.username.as_bytes()),
              &SerializedCell::Blob(user.password.as_bytes()),
            ],
          );
        });
      Ok(dataframe)
    };
    Ok(Box::pin(futures::stream::once(query)))
  }
}

fn get_scalar_string(value: &ColumnarValue) -> Option<String> {
  match value {
    ColumnarValue::Scalar(ScalarValue::Utf8(value)) => value.clone(),
    _ => None,
  }
}

fn scan_catalog_users(
  rows_iter: &mut HeapIterator,
  serializer: &Serializer,
) -> Result<Vec<(Vec<u8>, CatalogUser)>> {
  let mut users = vec![];
  while let Some((row_id, config)) = rows_iter.get()? {
    let user =
      serializer.deserialize::<CatalogUser>(config[0].as_bytes().unwrap())?;
    users.push((row_id.to_vec(), user));
    rows_iter.next();
  }
  Ok(users)
}
