use std::pin::Pin;
use std::sync::Arc;

use arenasql::datafusion::{
  ColumnarValue, DatafusionDataType as DfDataType, DatafusionField as Field,
  ScalarValue, Schema, SchemaRef, TaskContext,
};
use arenasql::execution::tablescan::HeapIterator;
use arenasql::execution::{
  convert_literals_to_columnar_values, CustomExecutionPlan,
  ExecutionPlanResponse, Privilege, SessionContext, Transaction,
};
use arenasql::runtime::RuntimeEnv;
use arenasql::schema::{
  DataFrame, DataType, OwnedSerializedCell, SerializedCell,
};
use arenasql::storage::Serializer;
use arenasql::{Error, Result};
use futures::Stream;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Expr;

use crate::schema::{ADMIN_USERNAME, APPS_USERNAME, SYSTEM_SCHEMA_NAME};
use crate::server::storage::ClusterStorageFactory;
use crate::server::ArenaSqlCluster;

// Note: store user/password config as a single column since it will be
// difficult to update this table once there are databases in prod
static CREATE_USERS_TABLE: &'static str =
  "CREATE TABLE IF NOT EXISTS arena_schema.users(config TEXT);";

pub fn schema() -> SchemaRef {
  SchemaRef::new(Schema::new(vec![
    Field::new("catalog", DfDataType::Utf8, false),
    Field::new("user", DfDataType::Utf8, false),
    Field::new("password", DfDataType::Utf8, false),
  ]))
}

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

    validate_username(&username)?;

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
    schema()
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<DataFrame>> + Send>>> {
    let session_context = create_admin_session_context_for_catalog(
      &self.transaction,
      &self.user.catalog,
    )?;

    let transaction = session_context.active_transaction();
    let user = self.user.clone();
    let query = async move {
      transaction.execute_sql(CREATE_USERS_TABLE).await?;

      let mut dataframe = DataFrame::with_capacity(
        10,
        vec![
          ("catalog".to_owned(), DataType::Text),
          ("user".to_owned(), DataType::Text),
          ("password".to_owned(), DataType::Text),
        ],
      );

      let handle = transaction.handle();
      let users_table = handle
        .get_table(SYSTEM_SCHEMA_NAME, "users")
        .ok_or_else(|| {
          Error::RelationDoesntExist(format!("{}.users", SYSTEM_SCHEMA_NAME))
        })?;
      let storage = handle.lock(false)?;
      let cols = vec![0];

      let mut rows_iter = HeapIterator::new(&storage, &users_table, &cols);
      let existing_users =
        scan_catalog_users(&mut rows_iter, &handle.serializer())?;

      for (row_id, user) in existing_users {
        if user.catalog == user.catalog && user.username == user.username {
          storage.delete_row(&users_table, &row_id)?;
        }
        dataframe.append_row(
          &row_id,
          &vec![
            &SerializedCell::String(user.catalog.as_str()),
            &SerializedCell::String(user.username.as_str()),
            &SerializedCell::String(user.password.as_str()),
          ],
        );
      }

      let row_id = storage.generate_next_row_id(&users_table)?;
      storage.insert_row(
        &users_table,
        &row_id,
        &vec![OwnedSerializedCell::Blob(
          handle.serializer().serialize(&user)?.into(),
        )],
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
    schema()
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<ExecutionPlanResponse> {
    let session_context = create_admin_session_context_for_catalog(
      &self.transaction,
      &self.catalog,
    )?;
    let transaction = session_context.active_transaction();
    let catalog = self.catalog.clone();
    let query = async move {
      let mut dataframe = DataFrame::with_capacity(
        10,
        vec![
          ("catalog".to_owned(), DataType::Text),
          ("user".to_owned(), DataType::Text),
          ("password".to_owned(), DataType::Text),
        ],
      );

      let handle = transaction.handle();
      let users_table = handle
        .get_table(SYSTEM_SCHEMA_NAME, "users")
        .ok_or_else(|| {
          Error::RelationDoesntExist(format!("{}.users", SYSTEM_SCHEMA_NAME))
        })?;
      let storage = handle.lock(false)?;
      let cols = vec![0];

      let mut rows_iter = HeapIterator::new(&storage, &users_table, &cols);
      let existing_users =
        scan_catalog_users(&mut rows_iter, &handle.serializer())?;

      existing_users
        .iter()
        .filter(|c| c.1.catalog == catalog)
        .for_each(|(row_id, user)| {
          dataframe.append_row(
            &row_id,
            &vec![
              &SerializedCell::String(user.catalog.as_str()),
              &SerializedCell::String(user.username.as_str()),
              &SerializedCell::String(user.password.as_str()),
            ],
          );
        });
      Ok(dataframe)
    };
    Ok(Box::pin(futures::stream::once(query)))
  }
}

fn create_admin_session_context_for_catalog(
  transaction: &Transaction,
  catalog: &str,
) -> Result<SessionContext> {
  let state = transaction.session_state();
  let cluster_storage_factory = state.borrow::<Arc<ClusterStorageFactory>>();
  let runtime = state.borrow::<Arc<RuntimeEnv>>();
  ArenaSqlCluster::create_session_context_using_cluster_storage(
    cluster_storage_factory.clone(),
    runtime.clone(),
    catalog,
    &ADMIN_USERNAME,
    Privilege::SUPER_USER,
  )
}

fn validate_username(username: &str) -> Result<()> {
  match username {
    ADMIN_USERNAME | APPS_USERNAME => Err(Error::ReservedWord(format!(
      "Can't use reserved username: {}",
      username
    ))),
    _ => Ok(()),
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
