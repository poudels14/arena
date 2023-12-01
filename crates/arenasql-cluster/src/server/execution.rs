use std::sync::Arc;

use arenasql::records::RecordBatch;
use arenasql::response::ExecutionResponse;
use futures::StreamExt;
use futures::TryStreamExt;
use pgwire::api::results::{FieldInfo, QueryResponse, Response, Tag};
use pgwire::api::ClientInfo;
use pgwire::error::PgWireResult;

use super::ArenaSqlCluster;
use crate::error::ArenaClusterError;
use crate::pgwire::statement::CommandString;
use crate::pgwire::ArenaQuery;
use crate::pgwire::{datatype, rowconverter};
use crate::to_query_execution_error;

impl ArenaSqlCluster {
  pub async fn execute_query<'a, C>(
    &self,
    client: &C,
    query: &ArenaQuery,
  ) -> PgWireResult<Vec<Response<'a>>>
  where
    C: ClientInfo,
  {
    let session = match &query.client {
      Some(client) => self.get_or_create_new_session(&client),
      None => self
        .session_store
        .get(client.metadata().get("session_id").unwrap())
        .ok_or_else(|| ArenaClusterError::InvalidConnection),
    }?;

    let mut results = Vec::with_capacity(query.stmts.len());
    for stmt in query.stmts.iter() {
      let txn = session
        .ctxt
        .begin_transaction()
        .map_err(|e| to_query_execution_error!(e))?;
      let response = txn
        .execute(stmt.clone())
        .await
        .map_err(|e| to_query_execution_error!(e))?;

      // Note: only commit non-query (eg: SELECT) transactions
      let response = match response.stmt_type {
        arenasql::response::Type::Query => Self::to_row_stream(response)?,
        _ => {
          let res = response
            .stream
            .try_collect::<Vec<RecordBatch>>()
            .await
            .map_err(|e| to_query_execution_error!(e))?;

          txn.commit().map_err(|e| to_query_execution_error!(e))?;
          Response::Execution(Tag::new_for_execution(
            stmt.command(),
            Some(res.iter().map(|b| b.num_rows()).sum()),
          ))
        }
      };
      results.push(response);
    }
    Ok(results)
  }

  fn to_row_stream<'a>(
    response: ExecutionResponse,
  ) -> PgWireResult<Response<'a>> {
    let fields: Vec<FieldInfo> = response
      .stream
      .schema()
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field))
      .collect();
    let schema = Arc::new(fields);

    let row_stream = response.stream.flat_map(|batch| {
      tokio_stream::iter(match batch {
        Ok(batch) => rowconverter::convert_to_rows(&batch),
        Err(e) => vec![Err(to_query_execution_error!(e))],
      })
    });
    Ok(Response::Query(QueryResponse::new(schema, row_stream)))
  }
}
