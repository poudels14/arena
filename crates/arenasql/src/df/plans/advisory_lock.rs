use std::ops::ControlFlow;
use std::pin::Pin;
use std::sync::Arc;

use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::execution::TaskContext;
use datafusion::logical_expr::expr::Placeholder;
use datafusion::logical_expr::{Expr, LogicalPlan};
use datafusion::scalar::ScalarValue;
use futures::{FutureExt, Stream};
use sqlparser::ast::{
  Expr as SQLExpr, Function, Statement as SQLStatement, Value, Visit, Visitor,
};

use crate::error::Error;
use crate::execution::{
  AdvisoryLock, AdvisoryLocks, CustomExecutionPlan, Transaction,
};
use crate::schema::DataFrame;
use crate::Result;

#[tracing::instrument(
  skip_all,
  fields(name = "advisory_lock"),
  level = "trace"
)]
pub fn extension(
  transaction: &Transaction,
  stmt: &SQLStatement,
) -> Result<Option<Arc<dyn CustomExecutionPlan>>> {
  let mut analyzer = AdvisoryLockAnalyzer::default();
  stmt.visit(&mut analyzer);
  if analyzer.func == "pg_advisory_lock" {
    Ok(Some(Arc::new(AdvisoryLockExecution {
      transaction: transaction.clone(),
      placeholder: analyzer.placeholder,
      lock_id: analyzer.lock_id,
      command: Command::Lock,
    })))
  } else if analyzer.func == "pg_advisory_unlock" {
    Ok(Some(Arc::new(AdvisoryLockExecution {
      transaction: transaction.clone(),
      placeholder: analyzer.placeholder,
      lock_id: analyzer.lock_id,
      command: Command::Unlock,
    })))
  } else {
    Ok(None)
  }
}

#[derive(Default, Debug)]
struct AdvisoryLockAnalyzer {
  func: String,
  placeholder: Option<Expr>,
  lock_id: Option<i64>,
}

impl Visitor for AdvisoryLockAnalyzer {
  type Break = ();
  fn pre_visit_expr(&mut self, expr: &SQLExpr) -> ControlFlow<Self::Break> {
    if let SQLExpr::Function(Function { name, .. }) = expr {
      self.func = name.0[0].value.clone();
    }
    if let SQLExpr::Value(value) = expr {
      match value {
        Value::Placeholder(_) => {
          self.placeholder = Some(Expr::Placeholder(Placeholder {
            id: "$1".to_string(),
            data_type: Some(DataType::Int64),
            metadata: Default::default(),
          }));
        }
        Value::Number(lock, _) => {
          if let Ok(lock_id) = lock.parse::<i64>() {
            self.lock_id = Some(lock_id);
          }
        }
        _ => {}
      }
    }
    ControlFlow::Continue(())
  }
}

#[derive(Clone)]
pub struct AdvisoryLockExecution {
  transaction: Transaction,
  placeholder: Option<Expr>,
  lock_id: Option<i64>,
  command: Command,
}

#[derive(Clone)]
enum Command {
  Lock,
  Unlock,
}

impl CustomExecutionPlan for AdvisoryLockExecution {
  fn schema(&self) -> SchemaRef {
    SchemaRef::new(Schema::new(Vec::<Field>::new()))
  }

  fn list_expressions(&self) -> Vec<Expr> {
    if let Some(placeholder) = &self.placeholder {
      vec![placeholder.clone()]
    } else {
      vec![]
    }
  }

  fn execute(
    &self,
    _partition: usize,
    _context: Arc<TaskContext>,
    exprs: Vec<Expr>,
    _inputs: Vec<LogicalPlan>,
  ) -> Result<Pin<Box<dyn Stream<Item = Result<DataFrame>> + Send>>> {
    let lock_id = self
      .lock_id
      .or_else(|| {
        if let Some(Expr::Literal(ScalarValue::Int64(value))) = exprs.get(0) {
          *value
        } else {
          None
        }
      })
      .ok_or(Error::InvalidQuery("INT8 expected".to_owned()))?;

    let transaction = self.transaction.clone();
    let command = self.command.clone();
    let fut = async move {
      let locks = {
        let state = transaction.session_state().read();
        state.borrow::<Arc<AdvisoryLocks>>().clone()
      };

      match command {
        Command::Lock => {
          let lock = locks.acquire_lock(lock_id).await?;
          let mut state = transaction.session_state().write();
          let prev = state.put(lock);
          if prev.is_some() {
            panic!("Holding more than one lock not supported yet")
          }
        }
        Command::Unlock => {
          let mut state = transaction.session_state().write();
          let lock = state.remove::<AdvisoryLock>();
          drop(lock);
          locks.release_lock(lock_id)?;
        }
      }

      Ok(DataFrame::empty())
    }
    .boxed();

    Ok(Box::pin(futures::stream::once(fut)))
  }
}
