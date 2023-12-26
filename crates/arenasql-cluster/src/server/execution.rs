use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use arenasql::ast::statement::StatementType;
use arenasql::datafusion::{LogicalPlan, ScalarValue};
use arenasql::pgwire::api::results::{
  FieldFormat, FieldInfo, QueryResponse, Response, Tag,
};
use arenasql::pgwire::api::ClientInfo;
use arenasql::pgwire::error::PgWireResult;
use arenasql::pgwire::messages::data::DataRow;
use arenasql::response::ExecutionResponse;
use futures::{Stream, StreamExt};
use sqlparser::ast::Statement;

use super::ArenaSqlCluster;
use crate::auth::{AuthHeader, AuthenticatedSession};
use crate::pgwire::ArenaQuery;
use crate::pgwire::{datatype, rowconverter};

impl ArenaSqlCluster {
  // TODO: to improve performance, instead of returning response from this
  // function, send the rows directly to client
  pub async fn execute_query<'a, C>(
    &self,
    client: &C,
    query: ArenaQuery,
    field_format: FieldFormat,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo,
  {
    let session = match &query.client {
      AuthHeader::None => self.get_client_session(client),
      header => self.get_or_create_new_session(client, &header),
    }?;

    // It seems like, in Postgres, all the statements in a single query
    // are run in the same transaction unless BEING/COMMIT/ROLLBACK is
    // explicity used
    let mut results = Vec::with_capacity(query.stmts.len());
    for stmt in query.stmts.into_iter() {
      let result = self
        .execute_plan(&session, stmt, None, None, field_format)
        .await
        .await?;
      results.push(result);
    }
    Ok(results)
  }

  pub(crate) async fn execute_plan<'a>(
    session: &AuthenticatedSession,
    stmt: Box<Statement>,
    logical_plan: Option<LogicalPlan>,
    params: Option<Vec<ScalarValue>>,
    field_format: FieldFormat,
  ) -> PgWireResult<Response<'a>> {
    let stmt_type = StatementType::from(stmt.as_ref());
    if stmt_type.is_begin() {
      session.begin_new_transaction()?;
      return Ok(Response::Execution(Tag::new_for_execution(
        stmt_type.to_string(),
        None,
      )));
    } else if stmt_type.is_commit() {
      let active_transaction = session.get_active_transaction();
      active_transaction.map(|t| t.commit()).transpose()?;
      session.clear_transaction();
      return Ok(Response::Execution(Tag::new_for_execution(
        stmt_type.to_string(),
        None,
      )));
    } else if stmt_type.is_rollback() {
      let active_transaction = session.get_active_transaction();
      active_transaction.map(|t| t.rollback()).transpose()?;
      session.clear_transaction();
      return Ok(Response::Execution(Tag::new_for_execution(
        stmt_type.to_string(),
        None,
      )));
    }

    let (transaction, chained_transaction) =
      session.get_active_transaction().map_or_else(
        || session.create_transaction().map(|t| (t, false)),
        |txn| Ok((txn.clone(), true)),
      )?;

    let logical_plan = match logical_plan {
      Some(logical_plan) => logical_plan,
      None => {
        transaction
          .create_verified_logical_plan(stmt.clone())
          .await?
      }
    };

    let final_logical_plan = match params {
      Some(param_values) => logical_plan
        .with_param_values(param_values)
        .map_err(|e| arenasql::Error::DataFusionError(e.into()))
        .expect(&format!(
          "Error replace_params_with_values at: {}:{}",
          file!(),
          line!()
        )),
      None => logical_plan,
    };

    let response = transaction
      .execute_logical_plan(&stmt_type, stmt, final_logical_plan)
      .await?;

    match stmt_type {
      // TODO: drop future/stream when connection drops?
      StatementType::Query | StatementType::Execute => Self::to_row_stream(
        response,
        field_format,
        StreamCompletionHook::new(Box::new(move || {
          // Commit the transaction if it's not a chained transaction
          // i.e. if it wasn't explicitly started by `BEGIN` command
          // Don't commit the SELECT statement's transaction since the
          // SELECT response stream will still need a valid transaction
          // when scanning rows
          if !chained_transaction {
            Ok(transaction.commit()?)
          } else {
            Ok(())
          }
        })),
      ),
      _ => {
        if !chained_transaction {
          transaction.commit()?;
        }
        Ok(Response::Execution(Tag::new_for_execution(
          stmt_type.to_string(),
          Some(response.get_modified_rows()),
        )))
      }
    }
  }

  fn to_row_stream<'a>(
    response: ExecutionResponse,
    field_format: FieldFormat,
    stream_completion_hook: StreamCompletionHook<'a>,
  ) -> PgWireResult<Response<'a>> {
    let response_stream = response.get_stream();
    let fields: Vec<FieldInfo> = response_stream
      .schema()
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field.as_ref(), field_format))
      .collect();
    let schema = Arc::new(fields);

    let rows_schema = schema.clone();
    let row_stream = response_stream.flat_map(move |batch| {
      futures::stream::iter(match batch {
        Ok(batch) => rowconverter::convert_to_rows(&schema, &batch),
        Err(e) => {
          vec![Err(arenasql::Error::DataFusionError(e.into()).into())]
        }
      })
    });
    Ok(Response::Query(QueryResponse::new(
      rows_schema,
      row_stream.chain(stream_completion_hook),
    )))
  }
}

struct StreamCompletionHook<'a> {
  hook: Option<Box<dyn (FnOnce() -> PgWireResult<()>) + Send + Sync + 'a>>,
}

impl<'a> StreamCompletionHook<'a> {
  pub fn new<F>(hook: Box<F>) -> Self
  where
    F: (FnOnce() -> PgWireResult<()>) + Send + Sync + 'a,
  {
    Self { hook: Some(hook) }
  }
}

impl<'a> Stream for StreamCompletionHook<'a> {
  type Item = PgWireResult<DataRow>;

  fn poll_next(
    mut self: Pin<&mut Self>,
    _cx: &mut Context<'_>,
  ) -> Poll<Option<Self::Item>> {
    let hook = self.hook.take().unwrap();
    if let Err(err) = hook() {
      Poll::Ready(Some(Err(err)))
    } else {
      Poll::Ready(None)
    }
  }

  fn size_hint(&self) -> (usize, Option<usize>) {
    (0, None)
  }
}
