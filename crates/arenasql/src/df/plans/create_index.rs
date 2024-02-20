use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use datafusion::arrow::array::UInt64Array;
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{Expr, LogicalPlan};
use datafusion::physical_plan::{DisplayAs, DisplayFormatType};
use derivative::Derivative;
use derive_builder::Builder;
use futures::StreamExt;
use sqlparser::ast::{Expr as SqlExpr, Statement as SQLStatement, Value};

use crate::df::providers::{get_schema_provider, get_table_ref};
use crate::execution::{CustomExecutionPlan, Transaction};
use crate::execution::{ExecutionPlanResponse, TransactionHandle};
use crate::schema::{
  DataFrame, IndexProvider, OwnedRow, Table, TableIndex, VectorMetric,
};
use crate::storage::{KeyValueGroup, StorageHandler};
use crate::{bail, table_rows_prefix_key, Error, Result};

macro_rules! bail_unsupported_query {
  ($msg:literal) => {
    return Err(Error::UnsupportedQuery(format!($msg)));
  };
}

macro_rules! invalid_query {
  ($msg:literal) => {
    Error::InvalidQuery(format!($msg))
  };
}

/// Returns a custom execution plan extension to create index
#[tracing::instrument(skip_all, fields(name = "create_index"), level = "trace")]
pub fn extension(
  transaction: &Transaction,
  stmt: &SQLStatement,
) -> Result<Option<Arc<dyn CustomExecutionPlan>>> {
  match stmt {
    SQLStatement::CreateIndex {
      name,
      table_name,
      columns,
      unique,
      if_not_exists,
      // Features below this are not supported
      concurrently,
      using,
      nulls_distinct,
      predicate,
      include,
    } => {
      if *concurrently {
        bail_unsupported_query!("`CONCURRENTLY` is not supported yet");
      } else if using.is_some() {
        let method = using.as_ref().unwrap();
        if method.value.as_str() != "hnsw" {
          bail_unsupported_query!("only `hnsw` index method supported");
        }
      } else if nulls_distinct.is_some() {
        bail_unsupported_query!("`NULLS NOT DISTINCT` is not supported yet");
      } else if predicate.is_some() {
        bail_unsupported_query!("Partial index is not supported yet");
      } else if !include.is_empty() {
        bail_unsupported_query!("`INCLUDE` is not supported yet");
      }

      let state = transaction.datafusion_context().state();
      let table_name = table_name.to_string();
      let table_ref = get_table_ref(&state, &table_name);
      let table_name = table_ref.table.as_ref().to_owned();

      let schema_provider = get_schema_provider(&state, &table_ref)?;

      if !schema_provider.table_exist(&table_name) {
        bail!(Error::RelationDoesntExist(table_name));
      }

      let table = transaction
        .handle()
        .get_table(&table_ref.schema, &table_name)
        .unwrap();
      let column_projection = columns
        .to_vec()
        .iter()
        .map(|c| c.to_string())
        .map(|col_name| {
          table
            .columns
            .iter()
            .position(|c| c.name == col_name)
            .ok_or_else(|| Error::ColumnDoesntExist(col_name.to_owned()))
        })
        .collect::<crate::Result<Vec<usize>>>()?;

      let create_index = CreateIndex {
        name: name.as_ref().map(|n| n.to_string()),
        catalog: table_ref.catalog.as_ref().into(),
        schema: table_ref.schema.as_ref().into(),
        table,
        columns: column_projection,
        unique: *unique,
        if_not_exists: *if_not_exists,
        using: using.as_ref().map(|using| using.value.clone()),
        predicate: predicate.clone(),
      };

      return Ok(Some(Arc::new(
        CreateIndexExecutionPlanBuilder::default()
          .transaction(transaction.handle().clone())
          .create_index(create_index)
          .build()
          .unwrap(),
      )));
    }
    _ => {}
  }
  Ok(None)
}

#[derive(Builder, Derivative)]
#[derivative(Debug)]
pub struct CreateIndexExecutionPlan {
  #[derivative(Debug = "ignore")]
  transaction: TransactionHandle,
  create_index: CreateIndex,
}

#[derive(Debug, Clone)]
pub struct CreateIndex {
  /// Index name
  pub name: Option<String>,
  pub catalog: Arc<str>,
  pub schema: Arc<str>,
  pub table: Arc<Table>,
  /// Column projection on the table
  pub columns: Vec<usize>,
  pub unique: bool,
  pub if_not_exists: bool,
  pub using: Option<String>,
  pub predicate: Option<SqlExpr>,
}

impl DisplayAs for CreateIndexExecutionPlan {
  fn fmt_as(
    &self,
    _t: DisplayFormatType,
    f: &mut fmt::Formatter,
  ) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl CustomExecutionPlan for CreateIndexExecutionPlan {
  fn schema(&self) -> SchemaRef {
    Arc::new(Schema::new(vec![Field::new(
      "count",
      DataType::UInt64,
      false,
    )]))
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
    _exprs: Vec<Expr>,
    _inputs: Vec<LogicalPlan>,
  ) -> crate::Result<ExecutionPlanResponse> {
    let create_index = self.create_index.clone();
    let transaction = self.transaction.clone();

    let response =
      DataFrame::from_arrays(vec![Arc::new(UInt64Array::from(vec![0]))]);
    let stream = futures::stream::once(async move {
      let CreateIndex {
        name: index_name,
        catalog,
        schema,
        table,
        columns,
        unique,
        if_not_exists,
        using,
        predicate,
      } = create_index;

      let mut table = table.as_ref().clone();
      let index_with_same_name_exist = index_name
        .as_ref()
        .map(|n| table.indexes.iter().any(|idx| idx.name == *n))
        .unwrap_or(false);

      if index_with_same_name_exist {
        if if_not_exists {
          return Ok(response);
        } else {
          bail!(Error::RelationAlreadyExists(index_name.unwrap()));
        }
      }

      let index_provider = match using.as_ref().map(|s| s.as_str()) {
        Some("hnsw") => {
          let options = predicate
            .and_then(|p| match p {
              SqlExpr::Struct { values, fields } => Some(
                fields
                  .into_iter()
                  .zip(values)
                  .map(|(field, value)| {
                    (
                      field.field_name.unwrap().value,
                      match value {
                        SqlExpr::Value(v) => v,
                        _ => unreachable!(),
                      },
                    )
                  })
                  .collect::<Vec<(String, Value)>>(),
              ),
              _ => None,
            })
            .ok_or_else(|| invalid_query!("Invalid hnsw index params"))?;

          let options = IndexOptions { options };
          let namespace_column = options
            .get_string("namespace")?
            .map(|name| {
              table
                .columns
                .iter()
                .position(|col| &col.name == name)
                .ok_or_else(|| Error::ColumnDoesntExist(name.to_owned()))
            })
            .transpose()?;
          IndexProvider::HNSWIndex {
            columns,
            metric: VectorMetric::from_str(
              &options.get_required_string("metric")?,
            )
            .map_err(|_| invalid_query!("invalid `metric` param"))?,
            m: options.get_required_number::<usize>("m")?,
            ef_construction: options
              .get_required_number::<usize>("ef_construction")?,
            ef: options.get_required_number::<usize>("ef")?,
            dim: options.get_required_number::<usize>("dim")?,
            retain_vectors: true,
            namespace_column,
          }
        }
        _ => IndexProvider::BasicIndex { columns, unique },
      };

      let table_lock = transaction
        .acquire_table_schema_write_lock(schema.as_ref(), &table.name)
        .await?;

      let storage_handler = transaction.lock(true)?;
      let index_id = storage_handler.get_next_table_index_id()?;
      let new_index = table.add_index(index_id, index_name, index_provider)?;

      storage_handler.put_table_schema(&catalog, &schema, &table)?;

      backfill_index_data(&storage_handler, &table, &new_index)?;

      transaction.hold_table_schema_lock(Arc::new(table), table_lock)?;
      Ok(response)
    })
    .boxed();

    Ok(Box::pin(stream))
  }
}

fn backfill_index_data(
  storage_handler: &StorageHandler,
  table: &Table,
  new_index: &TableIndex,
) -> Result<()> {
  let mut rows_iter = storage_handler
    .kv
    .scan_with_prefix(KeyValueGroup::Rows, &table_rows_prefix_key!(table.id))?;

  let table_row_prefix = table_rows_prefix_key!(table.id);
  while let Some((row_key, row_bytes)) = rows_iter.get() {
    let row_id_bytes = &row_key[table_row_prefix.len()..];
    let row = storage_handler
      .serializer
      .deserialize::<OwnedRow>(row_bytes)?;

    storage_handler.add_row_to_index(table, &new_index, row_id_bytes, &row)?;
    rows_iter.next();
  }
  Ok(())
}

struct IndexOptions {
  options: Vec<(String, Value)>,
}

impl IndexOptions {
  pub fn get_string(&self, key: &str) -> Result<Option<&String>> {
    self
      .get(key)
      .map(|value| match value {
        Value::SingleQuotedString(s) => Ok(s),
        _ => Err(Error::InvalidQuery(format!("invalid param `{}`", key))),
      })
      .transpose()
  }

  pub fn get_required_string(&self, key: &str) -> Result<&String> {
    match self.get_required(key)? {
      Value::SingleQuotedString(s) => Ok(s),
      _ => Err(Error::InvalidQuery(format!("invalid param `{}`", key))),
    }
  }

  pub fn get_required_number<T: FromStr>(&self, key: &str) -> Result<T> {
    match self.get_required(key)? {
      Value::Number(num, _) => {
        if let Ok(num) = num.parse::<T>() {
          return Ok(num);
        }
      }
      _ => {}
    }
    Err(Error::InvalidQuery(format!("invalid param `{}`", key)))
  }

  fn get_required(&self, key: &str) -> Result<&Value> {
    self
      .get(key)
      .ok_or_else(|| Error::InvalidQuery(format!("missing param `{}`", key)))
  }

  fn get(&self, key: &str) -> Option<&Value> {
    self
      .options
      .iter()
      .find(|opt| opt.0 == key)
      .map(|opt| &opt.1)
  }
}
