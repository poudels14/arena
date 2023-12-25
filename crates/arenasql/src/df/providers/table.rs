use std::any::Any;
use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::common::{Column, Constraints, SchemaError};
use datafusion::datasource::TableProvider as DfTableProvider;
use datafusion::error::{DataFusionError, Result};
use datafusion::execution::context::SessionState;
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_plan::insert::FileSinkExec;
use datafusion::physical_plan::{project_schema, ExecutionPlan};
use datafusion::sql::ResolvedTableReference;
use datafusion::sql::TableReference;
use getset::Getters;

use crate::df::plans::delete_rows::DeleteRowsExecutionPlanBuilder;
use crate::df::plans::insert_rows;
use crate::df::plans::scan_table::TableScanerBuilder;
use crate::df::plans::update_rows::UpdateRowsExecutionPlanBuilder;
use crate::execution::filter::Filter;
use crate::execution::TransactionHandle;
use crate::schema;

pub fn get_table_ref<'a>(
  state: &'a SessionState,
  table_name: &'a str,
) -> ResolvedTableReference<'a> {
  let table_ref = TableReference::parse_str(&table_name).to_owned();
  let catalog = &state.config_options().catalog;
  table_ref
    .clone()
    .resolve(&catalog.default_catalog, &catalog.default_schema)
}

#[derive(Getters)]
#[getset(get = "pub")]
pub struct TableProvider {
  table: Arc<schema::Table>,
  schema: SchemaRef,
  constraints: Option<Constraints>,
  transaction: TransactionHandle,
}

impl TableProvider {
  pub(crate) fn new(
    table: Arc<schema::Table>,
    transaction: TransactionHandle,
  ) -> Self {
    Self {
      schema: table.get_df_schema(),
      constraints: table.get_df_constraints(),
      table,
      transaction,
    }
  }

  pub(crate) async fn delete(
    &self,
    // scanner execution plan scans the table with appropriate filters
    // and returns the rows that needs to be deleted
    scanner: Arc<dyn ExecutionPlan>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    Ok(Arc::new(
      DeleteRowsExecutionPlanBuilder::default()
        .table(self.table.clone())
        .scanner(scanner)
        .transaction(self.transaction.clone())
        .build()
        .unwrap(),
    ))
  }

  pub(crate) async fn update(
    &self,
    // scanner execution plan scans the table with appropriate filters
    // and returns the rows that needs to be deleted
    scanner: Arc<dyn ExecutionPlan>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    Ok(Arc::new(
      UpdateRowsExecutionPlanBuilder::default()
        .table(self.table.clone())
        .scanner(scanner)
        .transaction(self.transaction.clone())
        .build()
        .unwrap(),
    ))
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

  fn supports_filters_pushdown(
    &self,
    filters: &[&Expr],
  ) -> Result<Vec<TableProviderFilterPushDown>> {
    filters
      .iter()
      .map(|expr| {
        let filter = Filter::for_table(&self.table, *expr)?;
        Ok(
          self
            .table
            .indexes
            .iter()
            .find_map(|index| {
              if filter.is_filter_pushdown_suported()
                && filter.is_supported_by_index(index)
              {
                Some(TableProviderFilterPushDown::Exact)
              } else {
                None
              }
            })
            .unwrap_or(TableProviderFilterPushDown::Unsupported),
        )
      })
      .collect::<Result<Vec<_>>>()
  }

  async fn scan(
    &self,
    _state: &SessionState,
    projection: Option<&Vec<usize>>,
    filters: &[Expr],
    limit: Option<usize>,
  ) -> Result<Arc<dyn ExecutionPlan>> {
    let projected_schema = project_schema(&self.schema, projection).unwrap();

    Ok(Arc::new(
      TableScanerBuilder::default()
        .table(self.table.clone())
        .projected_schema(projected_schema)
        .projection(
          projection
            .map(|p| p.to_vec())
            .unwrap_or_else(|| (0..self.table.columns.len()).collect()),
        )
        .transaction(self.transaction.clone())
        .filters(
          filters
            .iter()
            .map(|expr| Filter::for_table(&self.table, expr))
            .collect::<crate::Result<Vec<Filter>>>()?,
        )
        .limit(limit)
        .build()
        .unwrap(),
    ))
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
          field: Box::new(Column::new_unqualified(field.name().to_owned())),
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
      Arc::new(insert_rows::Sink {
        table: self.table.clone(),
        schema: sink_schema.clone(),
        transaction: self.transaction.clone(),
      }),
      sink_schema,
      None,
    )))
  }
}
