use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::common::{DFField, DFSchema};
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{
  AggregateUDF, Expr, ScalarUDF, TableSource, TypeSignature, WindowUDF,
};
use datafusion::physical_plan::ColumnarValue;
use datafusion::sql::planner::{ContextProvider, PlannerContext, SqlToRel};
use datafusion::sql::TableReference;
use futures::Stream;
use sqlparser::ast::Expr as SqlExpr;

use super::CustomExecutionPlan;
use crate::error::Error;
use crate::schema::DataFrame;
use crate::Result;

pub struct ScalarUdfExecutionPlan {
  schema: SchemaRef,
  udf: ScalarUDF,
  parameters: Vec<SqlExpr>,
}

impl ScalarUdfExecutionPlan {
  pub fn new(udf: ScalarUDF, parameters: Vec<SqlExpr>) -> Result<Self> {
    let schema = SchemaRef::new(Schema::new(vec![Field::new(
      "row",
      (udf.return_type)(&vec![])?.as_ref().clone(),
      true,
    )]));

    Ok(Self {
      schema,
      udf,
      parameters,
    })
  }
}

impl CustomExecutionPlan for ScalarUdfExecutionPlan {
  fn schema(&self) -> SchemaRef {
    self.schema.clone()
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<DataFrame>> + Send>>> {
    let ScalarUDF {
      name: func_name,
      fun: scalar_function,
      signature,
      ..
    } = &self.udf;
    let sql = SqlToRel::new(&SqlContextProvider {});
    let mut planner_ctxt = PlannerContext::new();

    let input_types = match &signature.type_signature {
      TypeSignature::Exact(types) => types,
      _ => {
        return Err(Error::InvalidDataType(format!(
          "Invalid argument for \"{}\"",
          func_name
        )))
      }
    };

    let args = self
      .parameters
      .iter()
      .zip(input_types)
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
          _ => Err(Error::InvalidDataType(format!(
            "Expected literal argument for \"{}\"",
            func_name
          ))),
        }
      })
      .collect::<Result<Vec<ColumnarValue>>>()?;

    let value = scalar_function(&args)?;
    let array = match value {
      ColumnarValue::Array(arr) => arr,
      ColumnarValue::Scalar(scalar) => scalar.to_array_of_size(1)?,
    };
    let dataframe = DataFrame::from_arrays(vec![array]);
    Ok(Box::pin(futures::stream::iter(vec![Ok(dataframe)])))
  }
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
