use std::sync::Arc;

use arenasql::ast::statement::StatementType;
use arenasql::datafusion::{LogicalPlan, ScalarValue};
use arenasql::pgwire::api::results::{
  FieldFormat, FieldInfo, QueryResponse, Response, Tag,
};
use arenasql::pgwire::error::PgWireResult;
use arenasql::response::ExecutionResponse;
use arenasql::sqlparser::ast::Statement;
use futures::StreamExt;

use super::ArenaSqlCluster;
use crate::auth::AuthenticatedSession;
use crate::pgwire::{datatype, rowconverter};

impl ArenaSqlCluster {
  pub(crate) async fn execute_plan<'a>(
    session: &AuthenticatedSession,
    stmt: Box<Statement>,
    logical_plan: Option<LogicalPlan>,
    params: Option<Vec<ScalarValue>>,
    field_format: FieldFormat,
  ) -> PgWireResult<Response<'a>> {
    let stmt_type = StatementType::from(stmt.as_ref());
    let session_context = session.context().clone();
    let response = session_context
      .execute_statement(stmt, logical_plan, params)
      .await?;

    match stmt_type {
      // TODO: drop future/stream when connection drops?
      StatementType::Query | StatementType::Execute => {
        Self::to_row_stream(response, field_format)
      }
      _ => Ok(Response::Execution(Tag::new_for_execution(
        stmt_type.to_string(),
        response.get_modified_rows(),
      ))),
    }
  }

  fn to_row_stream<'a>(
    response: ExecutionResponse,
    field_format: FieldFormat,
  ) -> PgWireResult<Response<'a>> {
    let response_stream = response.get_stream();
    let response_schema = response_stream.schema();
    let fields: Vec<FieldInfo> = response_schema
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field.as_ref(), field_format))
      .collect();
    let fields_schema = Arc::new(fields);

    let fields_schema_clone = fields_schema.clone();
    let row_stream = response_stream.flat_map(move |batch| {
      futures::stream::iter(match batch {
        Ok(batch) => rowconverter::convert_to_rows(
          &response_schema,
          &fields_schema_clone,
          &batch,
        ),
        Err(e) => {
          vec![Err(arenasql::Error::DataFusionError(e.into()).into())]
        }
      })
    });
    Ok(Response::Query(QueryResponse::new(
      fields_schema,
      row_stream,
    )))
  }
}
