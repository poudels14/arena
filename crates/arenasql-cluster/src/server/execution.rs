use std::sync::Arc;

use arenasql::ast::statement::StatementType;
use arenasql::response::ExecutionResponse;
use arenasql::Transaction;
use futures::StreamExt;
use pgwire::api::results::{FieldInfo, QueryResponse, Response, Tag};
use pgwire::api::ClientInfo;
use pgwire::error::PgWireResult;
use sqlparser::ast::Statement;

use super::ArenaSqlCluster;
use crate::auth::AuthenticatedSession;
use crate::pgwire::ArenaQuery;
use crate::pgwire::QueryClient;
use crate::pgwire::{datatype, rowconverter};

impl ArenaSqlCluster {
  // TODO: to improve performance, instead of returning response from this
  // function, send the rows directly to client
  pub async fn execute_query<'a, C>(
    &self,
    client: &C,
    query: ArenaQuery,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo,
  {
    let session = match &query.client {
      QueryClient::Unknown => self.get_client_session(client),
      client => self.get_or_create_new_session(&client),
    }?;

    // It seems like, in Postgres, all the statements in a single query
    // are run in the same transaction unless BEING/COMMIT/ROLLBACK is
    // explicity used
    let mut active_transaction = session.get_active_transaction();
    let mut results = Vec::with_capacity(query.stmts.len());
    for stmt in query.stmts.into_iter() {
      let result = self
        .execute_single_statement(&session, &mut active_transaction, stmt)
        .await?;
      results.push(result);
    }
    Ok(results)
  }

  pub async fn execute_single_statement<'a>(
    &self,
    session: &Arc<AuthenticatedSession>,
    active_transaction: &mut Option<Transaction>,
    stmt: Box<Statement>,
  ) -> PgWireResult<Response<'a>> {
    let stmt_ref = stmt.as_ref();
    Ok(if stmt_ref.is_begin() {
      *active_transaction = Some(session.begin_transaction()?);
      Response::Execution(Tag::new_for_execution(stmt_ref.get_type(), None))
    } else if stmt_ref.is_commit() {
      active_transaction.take().map(|t| t.commit()).transpose()?;
      session.clear_transaction();
      Response::Execution(Tag::new_for_execution(stmt_ref.get_type(), None))
    } else if stmt_ref.is_rollback() {
      active_transaction
        .take()
        .map(|t| t.rollback())
        .transpose()?;
      session.clear_transaction();
      Response::Execution(Tag::new_for_execution(stmt_ref.get_type(), None))
    } else {
      let (txn, chained) = active_transaction.as_ref().map_or_else(
        || session.create_transaction().map(|t| (t, false)),
        |txn| Ok((txn.clone(), true)),
      )?;
      let response = txn.execute(stmt.clone()).await?;

      // Commit the transaction if it's not a chained transaction
      // i.e. if it wasn't explicitly started by `BEGIN` command
      // Don't commit the SELECT statement's transaction since the
      // SELECT response stream will still need a valid transaction
      // when scanning rows
      if !chained && !stmt_ref.is_query() {
        txn.commit()?;
      }
      Self::map_to_pgwire_response(&stmt, response).await?
    })
  }

  pub(crate) async fn map_to_pgwire_response<'a>(
    stmt: &Statement,
    response: ExecutionResponse,
  ) -> PgWireResult<Response<'a>> {
    match response.stmt_type {
      // TODO: drop future/stream when connection drops?
      arenasql::response::Type::Query => Self::to_row_stream(response),
      _ => Ok(Response::Execution(Tag::new_for_execution(
        stmt.get_type(),
        Some(response.get_modified_rows()),
      ))),
    }
  }

  fn to_row_stream<'a>(
    response: ExecutionResponse,
  ) -> PgWireResult<Response<'a>> {
    let stream = response.get_stream();
    let fields: Vec<FieldInfo> = stream
      .schema()
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field.as_ref()))
      .collect();
    let schema = Arc::new(fields);

    let rows_schema = schema.clone();
    let row_stream = stream.flat_map(move |batch| {
      futures::stream::iter(match batch {
        Ok(batch) => rowconverter::convert_to_rows(&schema, &batch),
        Err(e) => {
          vec![Err(arenasql::Error::DataFusionError(e.into()).into())]
        }
      })
    });
    Ok(Response::Query(QueryResponse::new(rows_schema, row_stream)))
  }
}
