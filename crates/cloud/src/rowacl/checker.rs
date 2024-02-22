use std::collections::BTreeMap;
use std::ops::ControlFlow;

use anyhow::{anyhow, bail, Result};
use deno_core::Resource;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlparser::ast::{
  visit_expressions_mut, visit_statements_mut, BinaryOperator, Expr, Ident,
  SetExpr, Statement, TableAlias, TableFactor, TableWithJoins,
};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RowAclChecker {
  // list of acls per table
  acls_by_user_id: BTreeMap<
    // user id
    String,
    BTreeMap<
      // table id
      String,
      BTreeMap<AclType, Vec<Expr>>,
    >,
  >,
}

#[derive(
  Debug, Clone, PartialOrd, Ord, Eq, PartialEq, Serialize, Deserialize,
)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AclType {
  Select,
  Insert,
  Update,
  Delete,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RowAcl {
  user_id: String,
  table: String,
  r#type: AclType,
  // SQL filter expression; only the conditions
  // ex: `id > 10`, `id in (1, 2, 3)`, etc
  // Use wildcard `*` if filter doesn't apply, for eg, for Insert query
  // or to give full access
  filter: String,
}

impl Resource for RowAclChecker {}

impl RowAclChecker {
  pub fn from(acls: Vec<RowAcl>) -> Result<Self> {
    let mut acls_by_user_id = BTreeMap::new();
    acls
      .iter()
      .group_by(|acl| acl.user_id.clone())
      .into_iter()
      .for_each(|(user_id, user_acls)| {
        let acls_by_table = user_acls
          .group_by(|acl| acl.table.clone())
          .into_iter()
          .map(|(table, table_acls)| {
            let acls: BTreeMap<AclType, Vec<Expr>> = table_acls
              .group_by(|acl| acl.r#type.clone())
              .into_iter()
              .map(|(r#type, query_acls)| {
                let exprs = query_acls
                  .into_iter()
                  .filter_map(|acl| {
                    if acl.filter == "*" {
                      None
                    } else {
                      let mut expr_parser = Parser::new(&PostgreSqlDialect {})
                        .try_with_sql(&acl.filter)
                        .unwrap();
                      let filter_expr = expr_parser.parse_expr().unwrap();
                      Some(filter_expr)
                    }
                  })
                  .collect();
                (r#type, exprs)
              })
              .collect();
            (table, acls)
          })
          .collect::<BTreeMap<String, BTreeMap<AclType, Vec<Expr>>>>();
        acls_by_user_id.insert(user_id, acls_by_table);
      });
    let checker = RowAclChecker { acls_by_user_id };
    Ok(checker)
  }

  pub fn has_query_access(
    &self,
    user_id: &str,
    table: &str,
    r#type: AclType,
  ) -> bool {
    self
      .acls_by_user_id
      .get(user_id)
      .and_then(|acls| acls.get(table))
      .map(|table_acls| table_acls.contains_key(&r#type))
      .unwrap_or(false)
  }

  // returns none if the user doesn't have access
  pub fn apply_sql_filter(&self, user_id: &str, query: &str) -> Result<String> {
    let acls = self.acls_by_user_id.get(user_id);
    match acls {
      Some(acls) => apply_filter(acls, query),
      None => bail!("Doesn't have any access"),
    }
  }
}

fn apply_filter(
  acls_by_table: &BTreeMap<
    // table id
    String,
    BTreeMap<AclType, Vec<Expr>>,
  >,
  query: &str,
) -> Result<String> {
  let mut err = None;
  let mut table_counter = 0;
  let mut parsed = Parser::parse_sql(&PostgreSqlDialect {}, query)?;

  let mut build_table_alias_map =
    |table_with_joins: &mut TableWithJoins| -> Result<(String, Option<Ident>)> {
      let factor = &mut table_with_joins.relation;
      match factor {
        TableFactor::Table { name, alias, .. } => {
          if alias.is_none() {
            table_counter += 1;
            *alias = Some(TableAlias {
              name: Ident::new(format!("t{}", table_counter)),
              columns: vec![],
            });
          }

          Ok((
            name.0.last().unwrap().value.clone(),
            alias.as_ref().map(|a| a.name.clone()),
          ))
        }
        _ => Err(anyhow!("Unsupported query")),
      }
    };

  let build_selection_filter =
    |table: &str, alias: &Option<Ident>, acl_type: &AclType| {
      let filters = acls_by_table.get(table).and_then(|acls| {
        acls.get(acl_type).as_ref().map(|exprs| {
          if exprs.is_empty() {
            None
          } else {
            Some(exprs.iter().skip(1).fold(exprs[0].clone(), |agg, expr| {
              Expr::BinaryOp {
                left: Box::new(agg),
                op: BinaryOperator::Or,
                right: Box::new(expr.clone()),
              }
            }))
          }
        })
      });

      if filters.is_none() {
        bail!(format!("Doesn't have {:?} access", acl_type));
      }

      let mut filters = filters.unwrap();
      if let Some(alias) = alias {
        visit_expressions_mut(&mut filters, |expr| {
          match expr {
            Expr::Identifier(id) => {
              *expr = Expr::CompoundIdentifier(vec![
                alias.clone(),
                Ident::new(id.value.clone()),
              ]);
            }
            _ => {}
          }
          ControlFlow::<()>::Continue(())
        });
      }
      Ok(filters)
    };

  visit_statements_mut(&mut parsed, |stmt| {
    match stmt {
      Statement::Query(query) => {
        let query = query.as_mut();
        match query.body.as_mut() {
          SetExpr::Select(select) => {
            let select = select.as_mut();

            if select.from.len() > 1 {
              err = Some(anyhow!("Unsupported query"));
              return ControlFlow::Break(());
            }
            let table_alias = select
              .from
              .iter_mut()
              .map(|from| build_table_alias_map(from))
              .collect::<Result<Vec<(String, Option<Ident>)>>>();
            let table_alias = match table_alias {
              Ok(alias) => alias,
              Err(e) => {
                err = Some(e);
                return ControlFlow::Break(());
              }
            };

            let res = table_alias
              .iter()
              .map(|(table, alias)| {
                let filters =
                  build_selection_filter(&table, alias, &AclType::Select)?;

                // there won't be any filter on a wild card access
                if let Some(filters) = filters {
                  let selection = match &select.selection {
                    Some(selection) => Expr::BinaryOp {
                      left: Box::new(selection.clone()),
                      op: BinaryOperator::And,
                      right: Box::new(filters),
                    },
                    None => filters,
                  };
                  select.selection = Some(selection);
                }

                Ok(())
              })
              .collect::<Result<Vec<()>>>();
            if let Err(e) = res {
              err = Some(e);
              return ControlFlow::Break(());
            }
          }
          _ => {
            err = Some(anyhow!("Unsupported query"));
            return ControlFlow::Break(());
          }
        }
      }
      Statement::Insert { table_name, .. } => {
        let table = table_name.0.last().unwrap().value.clone();

        if !acls_by_table
          .get(&table)
          .map(|acls| acls.contains_key(&AclType::Insert))
          .unwrap_or(false)
        {
          err = Some(anyhow!("Doesn't have INSERT access"));
          return ControlFlow::Break(());
        }
      }
      Statement::Update {
        table, selection, ..
      } => {
        let table_alias = match build_table_alias_map(table) {
          Ok(alias) => alias,
          Err(e) => {
            err = Some(e);
            return ControlFlow::Break(());
          }
        };

        let filters = build_selection_filter(
          &table_alias.0,
          &table_alias.1,
          &AclType::Update,
        );
        if let Err(e) = filters {
          err = Some(e);
          return ControlFlow::Break(());
        }

        let filters = filters.unwrap();
        if let Some(filters) = filters {
          let filter_selection = match &selection {
            Some(selection) => Expr::BinaryOp {
              left: Box::new(selection.clone()),
              op: BinaryOperator::And,
              right: Box::new(filters),
            },
            None => filters,
          };
          *selection = Some(filter_selection);
        }
      }
      Statement::Delete {
        from, selection, ..
      } => {
        if from.len() > 1 {
          err = Some(anyhow!("Unsupported query"));
          return ControlFlow::Break(());
        }
        let table_alias = from
          .iter_mut()
          .map(|from| build_table_alias_map(from))
          .collect::<Result<Vec<(String, Option<Ident>)>>>();

        let table_alias = match table_alias {
          Ok(alias) => alias,
          Err(e) => {
            err = Some(e);
            return ControlFlow::Break(());
          }
        };

        let res = table_alias
          .iter()
          .map(|(table, alias)| {
            let filters =
              build_selection_filter(&table, alias, &AclType::Delete)?;

            // there won't be any filter on a wild card access
            if let Some(filters) = filters {
              let filter_selection = match &selection {
                Some(selection) => Expr::BinaryOp {
                  left: Box::new(selection.clone()),
                  op: BinaryOperator::And,
                  right: Box::new(filters),
                },
                None => filters,
              };
              *selection = Some(filter_selection);
            }

            Ok(())
          })
          .collect::<Result<Vec<()>>>();
        if let Err(e) = res {
          err = Some(e);
          return ControlFlow::Break(());
        }
      }
      _ => {}
    }
    ControlFlow::<()>::Continue(())
  });

  if let Some(err) = err {
    return Err(err);
  }
  Ok(parsed.iter().map(|stmt| stmt.to_string()).join("; "))
}

#[allow(unused_imports, dead_code)]
mod tests {
  use crate::rowacl::{AclType, RowAcl, RowAclChecker};

  fn acl(user_id: &str, table: &str, r#type: AclType, filter: &str) -> RowAcl {
    RowAcl {
      user_id: user_id.to_owned(),
      table: table.to_owned(),
      r#type,
      filter: filter.to_owned(),
    }
  }

  #[test]
  fn test_rowacl_check_query_access_on_same_table() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table_1",
      AclType::Select,
      "id = 1",
    )])
    .unwrap();
    assert!(checker.has_query_access("user_1", "table_1", AclType::Select));
    assert!(!checker.has_query_access("user_1", "table_1", AclType::Insert));
    assert!(!checker.has_query_access("user_1", "table_1", AclType::Update));
    assert!(!checker.has_query_access("user_1", "table_1", AclType::Delete));
  }

  #[test]
  fn test_rowacl_check_query_access_on_different_user_same_table() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table_1",
      AclType::Select,
      "id = 1",
    )])
    .unwrap();
    assert!(!checker.has_query_access("user_2", "table_1", AclType::Select));
    assert!(!checker.has_query_access("user_2", "table_1", AclType::Insert));
    assert!(!checker.has_query_access("user_2", "table_1", AclType::Update));
    assert!(!checker.has_query_access("user_2", "table_1", AclType::Delete));
  }

  #[test]
  fn test_rowacl_check_query_access_on_different_table() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table_1",
      AclType::Select,
      "id = 1",
    )])
    .unwrap();
    assert!(!checker.has_query_access("user_1", "table_2", AclType::Select));
    assert!(!checker.has_query_access("user_1", "table_2", AclType::Insert));
    assert!(!checker.has_query_access("user_1", "table_2", AclType::Update));
    assert!(!checker.has_query_access("user_1", "table_2", AclType::Delete));
  }

  #[test]
  fn test_rowacl_apply_eq_filter_simple_select() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table1",
      AclType::Select,
      "id = 1",
    )])
    .unwrap();

    // without alias, should add alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
      "SELECT * FROM table1 AS t1 WHERE t1.id = 1".to_owned()
    );

    // with alias, should preserve alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1 t")
        .unwrap(),
      "SELECT * FROM table1 AS t WHERE t.id = 1".to_owned()
    );

    // with quoted table name
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM \"table1\" t")
        .unwrap(),
      "SELECT * FROM \"table1\" AS t WHERE t.id = 1".to_owned()
    );

    // with quoted alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM \"table1\" \"t\"")
        .unwrap(),
      "SELECT * FROM \"table1\" AS \"t\" WHERE \"t\".id = 1".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_combined_filter_simple_select() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table1",
      AclType::Select,
      "id = 1 AND age < 10",
    )])
    .unwrap();

    // without alias, should add alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
      "SELECT * FROM table1 AS t1 WHERE t1.id = 1 AND t1.age < 10".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_filter_on_select_query_wildcard_filter() {
    let checker =
      RowAclChecker::from(vec![acl("user_1", "table1", AclType::Select, "*")])
        .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
      "SELECT * FROM table1 AS t1".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_two_eq_filters_single_select() {
    let checker = RowAclChecker::from(vec![
      acl("user_1", "table1", AclType::Select, "id = 1"),
      acl("user_1", "table1", AclType::Select, "id = 2"),
      acl("user_1", "table2", AclType::Select, "id in (69, 420)"),
    ])
    .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
      "SELECT * FROM table1 AS t1 WHERE t1.id = 1 OR t1.id = 2".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_three_different_filters_single_select() {
    let checker = RowAclChecker::from(vec![
      acl("user_1", "table1", AclType::Select, "name ilike 'arena%'"),
      acl("user_1", "table1", AclType::Select, "age > 10"),
      acl("user_1", "table1", AclType::Select, "id in (69, 420)"),
    ])
    .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
        "SELECT * FROM table1 AS t1 WHERE t1.name ILIKE 'arena%' OR t1.age > 10 OR t1.id IN (69, 420)".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_filter_on_select_without_access() {
    let checker = RowAclChecker::from(vec![
      acl("user_1", "table1", AclType::Insert, "*"),
      acl("user_1", "table1", AclType::Update, "*"),
      acl("user_1", "table1", AclType::Delete, "*"),
    ])
    .unwrap();

    let res = checker.apply_sql_filter("user_1", "SELECT * FROM table1");
    assert!(res.is_err());
  }

  #[test]
  fn test_rowacl_apply_filter_on_insert_query_with_access() {
    let checker =
      RowAclChecker::from(vec![acl("user_1", "table1", AclType::Insert, "*")])
        .unwrap();

    let res =
      checker.apply_sql_filter("user_1", "INSERT INTO table1 VALUES(1)");
    assert!(res.is_ok());
  }

  #[test]
  fn test_rowacl_apply_filter_on_insert_query_without_access_errors() {
    let checker =
      RowAclChecker::from(vec![acl("user_1", "table1", AclType::Select, "*")])
        .unwrap();

    let res =
      checker.apply_sql_filter("user_1", "INSERT INTO table1 VALUES(1)");
    assert!(res.is_err());
  }

  #[test]
  fn test_rowacl_apply_filter_on_update_query_wildcard_filter() {
    let checker =
      RowAclChecker::from(vec![acl("user_1", "table1", AclType::Update, "*")])
        .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter("user_1", "UPDATE table1 SET name = 'my name'")
        .unwrap(),
      "UPDATE table1 AS t1 SET name = 'my name'".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_filter_on_update_query() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table1",
      AclType::Update,
      "id = 1",
    )])
    .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter(
          "user_1",
          "UPDATE table1 SET name = 'my name' WHERE id = 1111"
        )
        .unwrap(),
      "UPDATE table1 AS t1 SET name = 'my name' WHERE id = 1111 AND t1.id = 1"
        .to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_filter_on_delete_query_wildcard_filter() {
    let checker =
      RowAclChecker::from(vec![acl("user_1", "table1", AclType::Delete, "*")])
        .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter("user_1", "DELETE FROM table1 WHERE name = 'my name'")
        .unwrap(),
      "DELETE FROM table1 AS t1 WHERE name = 'my name'".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_filter_on_delete_query() {
    let checker = RowAclChecker::from(vec![acl(
      "user_1",
      "table1",
      AclType::Delete,
      "id = 99",
    )])
    .unwrap();

    assert_eq!(
      checker
        .apply_sql_filter(
          "user_1",
          "DELETE FROM table1 WHERE name = 'my name' AND id = 1111"
        )
        .unwrap(),
        "DELETE FROM table1 AS t1 WHERE name = 'my name' AND id = 1111 AND t1.id = 99"
          .to_owned()
    );
  }
}
