use std::collections::HashMap;
use std::sync::Arc;

use datafusion::arrow::datatypes::DataType;
use datafusion::common::tree_node::{Transformed, TreeNode};
use datafusion::common::{internal_err, plan_err, DFField, DFSchema};
use datafusion::config::ConfigOptions;
use datafusion::error::{DataFusionError, Result as DataFusionResult};
use datafusion::logical_expr::expr::Placeholder;
use datafusion::logical_expr::{
  AggregateUDF, Expr, ScalarUDF, TableSource, WindowUDF,
};
use datafusion::physical_plan::ColumnarValue;
use datafusion::scalar::ScalarValue;
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

// credit: datafusion
// copied from datafusion
pub fn replace_placeholders_with_values(
  exprs: Vec<Expr>,
  param_values: &[ScalarValue],
) -> DataFusionResult<Vec<Expr>> {
  exprs
    .into_iter()
    .map(|expr| {
      expr.transform(&|expr| {
        match &expr {
          Expr::Placeholder(Placeholder { id, data_type, .. }) => {
            if id.is_empty() || id == "$0" {
              return plan_err!("Empty placeholder id");
            }
            // convert id (in format $1, $2, ..) to idx (0, 1, ..)
            let idx = id[1..].parse::<usize>().map_err(|e| {
              DataFusionError::Internal(format!(
                "Failed to parse placeholder id: {e}"
              ))
            })?
              - 1;
            // value at the idx-th position in param_values should be the value for the placeholder
            let value = param_values.get(idx).ok_or_else(|| {
              DataFusionError::Internal(format!(
                "No value found for placeholder with id {id}"
              ))
            })?;
            // check if the data type of the value matches the data type of the placeholder
            if Some(value.data_type()) != *data_type {
              return internal_err!(
                "Placeholder value type mismatch: expected {:?}, got {:?}",
                data_type,
                value.data_type()
              );
            }
            // Replace the placeholder with the value
            Ok(Transformed::Yes(Expr::Literal(value.clone())))
          }
          _ => Ok(Transformed::No(expr)),
        }
      })
    })
    .collect()
}

pub fn convert_sql_params_to_df_expr(
  types: &Vec<DataType>,
  parameters: &Vec<SqlExpr>,
) -> Result<Vec<Expr>> {
  let sql = SqlToRel::new(&SqlContextProvider {});
  let mut planner_ctxt = PlannerContext::new();

  parameters
    .iter()
    .zip(types)
    .enumerate()
    .map(|(idx, (param, data_type))| {
      let mut expr = sql.sql_to_expr(
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

      match expr {
        Expr::Placeholder(ref mut placeholder) => {
          // If the placeholder datatype wasn't set, set it here
          if placeholder.data_type.is_none() {
            placeholder.data_type = Some(data_type.clone())
          }
        }
        _ => {}
      }
      Ok(expr)
    })
    .collect::<Result<Vec<Expr>>>()
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
