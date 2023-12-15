use std::sync::Arc;

use arenasql::records::RecordBatch;
use arenasql::response::ExecutionResponse;
use futures::StreamExt;
use futures::TryStreamExt;
use pgwire::api::results::{FieldInfo, QueryResponse, Response, Tag};
use pgwire::api::ClientInfo;
use pgwire::error::PgWireResult;
use sqlparser::ast::Statement;

use super::ArenaSqlCluster;
use crate::pgwire::statement::SqlCommand;
use crate::pgwire::ArenaQuery;
use crate::pgwire::QueryClient;
use crate::pgwire::{datatype, rowconverter};
use crate::query_execution_error;

impl ArenaSqlCluster {
  // TODO: to improve performance, instead of returning response from this
  // function, send the rows directly to client
  pub async fn execute_query<'a, C>(
    &self,
    client: &C,
    query: &ArenaQuery,
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
    for stmt in query.stmts.iter() {
      let stmt_ref = stmt.as_ref();
      let result = if stmt_ref.is_begin() {
        active_transaction =
          session.set_active_transaction(session.context.begin_transaction()?);
        Response::Execution(Tag::new_for_execution(stmt_ref.command(), None))
      } else if stmt_ref.is_commit() {
        active_transaction.take().map(|t| t.commit()).transpose()?;
        session.clear_transaction();
        Response::Execution(Tag::new_for_execution(stmt_ref.command(), None))
      } else if stmt_ref.is_rollback() {
        active_transaction
          .take()
          .map(|t| t.rollback())
          .transpose()?;
        session.clear_transaction();
        Response::Execution(Tag::new_for_execution(stmt_ref.command(), None))
      } else {
        let (txn, chained) = active_transaction.as_ref().map_or_else(
          || session.context.begin_transaction().map(|t| (t, false)),
          |txn| Ok((txn.clone(), true)),
        )?;
        let response = txn.execute(stmt.clone()).await?;

        // Commit the transaction if it's not a chained transaction
        // i.e. if it wasn't explicitly started by `BEGIN` command
        if !chained {
          txn.commit()?;
        }
        Self::map_to_pgwire_response(&stmt, response).await?
      };
      results.push(result);
    }
    Ok(results)
  }

  pub(crate) async fn map_to_pgwire_response<'a>(
    stmt: &Statement,
    response: ExecutionResponse,
  ) -> PgWireResult<Response<'a>> {
    // Note: only commit non-query (eg: SELECT) transactions
    match response.stmt_type {
      // TODO: drop future/stream when connection drops?
      arenasql::response::Type::Query => Self::to_row_stream(response),
      _ => {
        let res = response
          .stream
          .try_collect::<Vec<RecordBatch>>()
          .await
          .map_err(|e| arenasql::Error::DataFusionError(e.into()))?;

        Ok(Response::Execution(Tag::new_for_execution(
          stmt.command(),
          Some(res.iter().map(|b| b.num_rows()).sum()),
        )))
      }
    }
  }

  fn to_row_stream<'a>(
    response: ExecutionResponse,
  ) -> PgWireResult<Response<'a>> {
    let fields: Vec<FieldInfo> = response
      .stream
      .schema()
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field.as_ref()))
      .collect();
    let schema = Arc::new(fields);

    let row_stream = response.stream.flat_map(|batch| {
      futures::stream::iter(match batch {
        Ok(batch) => rowconverter::convert_to_rows(&batch),
        Err(e) => vec![Err(query_execution_error!(e.to_string()))],
      })
    });
    Ok(Response::Query(QueryResponse::new(schema, row_stream)))
  }
}
