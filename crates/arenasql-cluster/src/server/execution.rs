use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use arenasql::ast::statement::StatementType;
use arenasql::execution::Transaction;
use arenasql::response::ExecutionResponse;
use futures::{Stream, StreamExt};
use pgwire::api::results::{FieldInfo, QueryResponse, Response, Tag};
use pgwire::api::ClientInfo;
use pgwire::error::PgWireResult;
use pgwire::messages::data::DataRow;
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
    let stmt_type = StatementType::from(stmt.as_ref());
    Ok(if stmt_type.is_begin() {
      *active_transaction = Some(session.begin_transaction()?);
      Response::Execution(Tag::new_for_execution(stmt_type.to_string(), None))
    } else if stmt_type.is_commit() {
      active_transaction.take().map(|t| t.commit()).transpose()?;
      session.clear_transaction();
      Response::Execution(Tag::new_for_execution(stmt_type.to_string(), None))
    } else if stmt_type.is_rollback() {
      active_transaction
        .take()
        .map(|t| t.rollback())
        .transpose()?;
      session.clear_transaction();
      Response::Execution(Tag::new_for_execution(stmt_type.to_string(), None))
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
      let transaction_to_commit = if !chained { Some(txn) } else { None };
      Self::map_to_pgwire_response(&stmt_type, response, transaction_to_commit)
        .await?
    })
  }

  pub(crate) async fn map_to_pgwire_response<'a>(
    stmt_type: &StatementType,
    response: ExecutionResponse,
    // If transaction is not None, it will be committed
    // after the row stream is complete
    transaction: Option<Transaction>,
  ) -> PgWireResult<Response<'a>> {
    match stmt_type {
      // TODO: drop future/stream when connection drops?
      StatementType::Query | StatementType::Execute => {
        Self::to_row_stream(response, transaction)
      }
      _ => {
        if let Some(transaction) = transaction {
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
    // Transaction to commit at the end of the stream
    transaction: Option<Transaction>,
  ) -> PgWireResult<Response<'a>> {
    let response_stream = response.get_stream();
    let fields: Vec<FieldInfo> = response_stream
      .schema()
      .fields
      .iter()
      .map(|field| datatype::to_field_info(field.as_ref()))
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
      row_stream.chain(StreamCompletionHook::new(Box::new(|| {
        if let Some(txn) = transaction {
          Ok(txn.commit()?)
        } else {
          Ok(())
        }
      }))),
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
