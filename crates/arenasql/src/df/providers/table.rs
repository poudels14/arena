use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::{Field, Schema, SchemaRef};
use datafusion::common::{Column, Constraints, SchemaError};
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::SessionState;
use datafusion::logical_expr::{Expr, TableType};
use datafusion::physical_plan::insert::FileSinkExec;
use datafusion::physical_plan::{project_schema, ExecutionPlan};

use super::super::scan;
use crate::df::insert;
use crate::schema;
use crate::storage::Transaction;

pub struct TableProvider {
  table: Arc<schema::Table>,
  schema: SchemaRef,
  constraints: Option<Constraints>,
  transaction: Transaction,
}

impl TableProvider {
  pub(super) fn new(table: schema::Table, transaction: Transaction) -> Self {
    let fields: Vec<Field> =
      table.columns.iter().map(|col| col.to_field()).collect();
    let schema_ref = SchemaRef::new(Schema::new(fields));
    Self {
      table: Arc::new(table),
      schema: schema_ref,
      constraints: None,
      transaction,
    }
  }
}

#[async_trait]
impl DfTableProvider for TableProvider {
  fn as_any(&self) -> &dyn Any {
    self
  }

  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }

  fn constraints(&self) -> Option<&Constraints> {
    self.constraints.as_ref()
  }

  fn table_type(&self) -> TableType {
    TableType::Base
  }

  fn get_table_definition(&self) -> Option<&str> {
    None
  }

  async fn scan(
    &self,
    _state: &SessionState,
    projection: Option<&Vec<usize>>,
    filters: &[Expr],
    limit: Option<usize>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    let projected_schema = project_schema(&self.schema, projection).unwrap();
    Ok(Arc::new(scan::TableScaner {
      table: self.table.clone(),
      projection: projection
        .map(|p| p.to_vec())
        .unwrap_or_else(|| (0..self.table.columns.len()).collect()),
      projected_schema,
      transaction: self.transaction.clone(),
      filters: filters.to_vec(),
      limit,
    }))
  }

  async fn insert_into(
    &self,
    _state: &SessionState,
    input: Arc<dyn ExecutionPlan>,
    _overwrite: bool,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    let sink_schema = input
      .schema()
      .fields
      .iter()
      .map(|field| {
        let idx = self
          .table
          .columns
          .iter()
          .find(|c| c.name == *field.name())
          .map(|c| c.id as usize);
        idx.ok_or(DataFusionError::SchemaError(SchemaError::FieldNotFound {
          field: Box::new(Column {
            name: field.name().to_owned(),
            relation: None,
          }),
          valid_fields: vec![],
        }))
      })
      .collect::<Result<Vec<usize>>>()
      .and_then(|projection| {
        self
          .schema
          .project(&projection)
          .map(|s| Arc::new(s))
          .map_err(|e| DataFusionError::ArrowError(e))
      })?;

    Ok(Arc::new(FileSinkExec::new(
      input,
      Arc::new(insert::Sink {
        table: self.table.clone(),
        schema: sink_schema.clone(),
        transaction: self.transaction.clone(),
      }),
      sink_schema,
    )))
  }
}
