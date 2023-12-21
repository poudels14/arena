use std::collections::HashMap;
use std::sync::Arc;

use datafusion::arrow::datatypes::DataType;
use datafusion::common::{DFField, DFSchema};
use datafusion::config::ConfigOptions;
use datafusion::logical_expr::{
  AggregateUDF, Expr, ScalarUDF, TableSource, WindowUDF,
};
use datafusion::physical_plan::ColumnarValue;
use datafusion::sql::planner::{ContextProvider, PlannerContext, SqlToRel};
use datafusion::sql::TableReference;
use sqlparser::ast::Expr as SqlExpr;

use crate::error::Error;
use crate::Result;

pub fn convert_literals_to_columnar_values(
  types: &Vec<DataType>,
  parameters: &Vec<SqlExpr>,
) -> Result<Vec<ColumnarValue>> {
  let sql = SqlToRel::new(&SqlContextProvider {});
  let mut planner_ctxt = PlannerContext::new();

  parameters
    .iter()
    .zip(types)
    .enumerate()
    .map(|(idx, (param, data_type))| {
      let lit = sql.sql_to_expr(
        param.clone(),
        &DFSchema::new_with_metadata(
          vec![DFField::new_unqualified(
            &idx.to_string(),
            data_type.clone(),
            true,
          )],
          HashMap::new(),
        )?,
        &mut planner_ctxt,
      )?;

      match lit {
        Expr::Literal(v) => match v.data_type() == *data_type {
          false => Err(Error::InvalidDataType(format!(
            "Expected data type \"{}\" but got \"{}\" at argument {}",
            data_type,
            v.data_type(),
            idx
          ))),
          true => Ok(ColumnarValue::Scalar(v)),
        },
        _ => Err(Error::InvalidDataType(format!("Expected literal value",))),
      }
    })
    .collect::<Result<Vec<ColumnarValue>>>()
}

pub(self) struct SqlContextProvider {}

impl ContextProvider for SqlContextProvider {
  fn get_aggregate_meta(&self, _name: &str) -> Option<Arc<AggregateUDF>> {
    unimplemented!()
  }

  fn get_function_meta(&self, _name: &str) -> Option<Arc<ScalarUDF>> {
    unimplemented!()
  }

  fn get_table_source(
    &self,
    _name: TableReference,
  ) -> datafusion::error::Result<Arc<dyn TableSource>> {
    unimplemented!()
  }

  fn get_variable_type(&self, _variable_names: &[String]) -> Option<DataType> {
    unimplemented!()
  }

  fn get_window_meta(&self, _name: &str) -> Option<Arc<WindowUDF>> {
    unimplemented!()
  }

  fn options(&self) -> &ConfigOptions {
    unimplemented!()
  }
}
