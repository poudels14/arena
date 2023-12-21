use std::pin::Pin;
use std::sync::Arc;

use datafusion::arrow::datatypes::{Field, Schema, SchemaRef};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::{ScalarUDF, TypeSignature};
use datafusion::physical_plan::ColumnarValue;
use futures::Stream;
use sqlparser::ast::Expr;

use super::super::CustomExecutionPlan;
use super::convert_literals_to_columnar_values;
use crate::error::Error;
use crate::schema::DataFrame;
use crate::Result;

pub struct ScalarUdfExecutionPlan {
  schema: SchemaRef,
  udf: ScalarUDF,
  parameters: Vec<Expr>,
}

impl ScalarUdfExecutionPlan {
  pub fn new(udf: ScalarUDF, parameters: Vec<Expr>) -> Result<Self> {
    let schema = SchemaRef::new(Schema::new(vec![Field::new(
      "data",
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
    let input_types = match &signature.type_signature {
      TypeSignature::Exact(types) => types,
      _ => {
        return Err(Error::InvalidDataType(format!(
          "Invalid argument for \"{}\"",
          func_name
        )))
      }
    };

    let args =
      convert_literals_to_columnar_values(&input_types, &self.parameters)?;
    let value = scalar_function(&args)?;
    let array = match value {
      ColumnarValue::Array(arr) => arr,
      ColumnarValue::Scalar(scalar) => scalar.to_array_of_size(1)?,
    };
    let dataframe = DataFrame::from_arrays(vec![array]);
    Ok(Box::pin(futures::stream::iter(vec![Ok(dataframe)])))
  }
}
