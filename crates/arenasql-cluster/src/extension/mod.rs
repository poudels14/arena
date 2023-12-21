use std::collections::BTreeMap;
use std::sync::Arc;

use arenasql::datafusion::ScalarUDF;
use arenasql::execution::{
  CustomExecutionPlan, ScalarUdfExecutionPlan, SessionState, Transaction,
};
use arenasql::Result;
use once_cell::sync::Lazy;
use sqlparser::ast::Statement;

use self::catalog_users::SetCatalogUserCredentials;
use crate::extension::catalog_users::ListCatalogUserCredentials;

pub(crate) mod catalog_users;

/// Returns a custom admin execution plan extension
pub fn admin_exetension(
  _state: &SessionState,
  transaction: &Transaction,
  stmt: &Statement,
) -> Result<Option<Arc<dyn CustomExecutionPlan>>> {
  match stmt {
    Statement::Execute { name, parameters } => {
      let extension = SCALAR_EXTENSIONS.get(&name.value).map(|f| f.clone());
      if let Some(udf) = extension {
        return Ok(Some(Arc::new(ScalarUdfExecutionPlan::new(
          udf,
          parameters.clone(),
        )?)));
      }

      match name.value.as_str() {
        "arena_set_catalog_user_credential" => {
          return Ok(Some(Arc::new(SetCatalogUserCredentials::new(
            transaction.clone(),
            parameters,
          )?)))
        }
        "arena_list_catalog_users" => {
          return Ok(Some(Arc::new(ListCatalogUserCredentials::new(
            transaction.clone(),
            parameters,
          )?)))
        }
        _ => {}
      }
    }
    _ => {}
  }
  Ok(None)
}

pub const SCALAR_EXTENSIONS: Lazy<BTreeMap<String, ScalarUDF>> =
  Lazy::new(|| BTreeMap::from_iter(vec![]));
