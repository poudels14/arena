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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowAclChecker {
  // list of acls per table
  acls_by_user_id: BTreeMap<
    // user id
    // `public` user id is reserved for non-logged in users, useful for sites
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
pub enum AclType {
  Select,
  Insert,
  Update,
  Delete,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RowAcl {
  pub user_id: String,
  pub table: String,
  pub r#type: AclType,
  // SQL filter expression; only the conditions
  // ex: `id > 10`, `id in (1, 2, 3)`, etc
  // Use wildcard `*` if filter doesn't apply, for eg, for Insert query
  // or to give full access
  pub filter: String,
}

impl Resource for RowAclChecker {}

impl RowAclChecker {
  pub fn from(acls: Vec<RowAcl>) -> Result<Self> {
    let mut checker = RowAclChecker {
      acls_by_user_id: BTreeMap::new(),
    };
    checker.set_acls(acls);
    Ok(checker)
  }

  pub fn set_acls(&mut self, acls: Vec<RowAcl>) {
    self.acls_by_user_id.clear();
    acls
      .iter()
      .sorted_by(|u1, u2| Ord::cmp(&u1.user_id, &u2.user_id))
      .group_by(|acl| acl.user_id.clone())
      .into_iter()
      .for_each(|(user_id, user_acls)| {
        let acls_by_table = user_acls
          .group_by(|acl| acl.table.clone())
          .into_iter()
          .sorted_by(|(t1, _), (t2, _)| Ord::cmp(&t1, &t2))
          .map(|(table, table_acls)| {
            let acls: BTreeMap<AclType, Vec<Expr>> = table_acls
              .group_by(|acl| acl.r#type.clone())
              .into_iter()
              .map(|(r#type, query_acls)| {
                let exprs = query_acls
                  .into_iter()
                  .map(|acl| {
                    if acl.filter == "*" {
                      vec![]
                    } else {
                      let mut expr_parser = Parser::new(&PostgreSqlDialect {})
                        .try_with_sql(&acl.filter)
                        .unwrap();
                      let filter_expr = expr_parser.parse_expr().unwrap();
                      vec![filter_expr]
                    }
                  })
                  .flatten()
                  .collect();
                (r#type, exprs)
              })
              .collect();
            (table, acls)
          })
          .collect::<BTreeMap<String, BTreeMap<AclType, Vec<Expr>>>>();
        self.acls_by_user_id.insert(user_id, acls_by_table);
      });
  }

  /// Returns whether the user has any type of access
  /// This can be used to check if the user can access an app
  pub fn has_any_access(&self, user_id: &str) -> bool {
    self
      .acls_by_user_id
      .get(user_id)
      .map(|acls| !acls.is_empty())
      .unwrap_or(false)
  }

  pub fn has_query_access(
    &self,
    user_id: &str,
    table: &str,
    r#type: AclType,
  ) -> bool {
    let user_acls = self.acls_by_user_id.get(user_id);

    user_acls
      .and_then(|acls| {
        acls
          // widlcard table access
          .get("*")
          .map(|table_acls| table_acls.contains_key(&r#type))
      })
      .unwrap_or(false)
      || user_acls
        .and_then(|acls| acls.get(table))
        .map(|table_acls| table_acls.contains_key(&r#type))
        .unwrap_or(false)
  }

  // returns none if the user doesn't have access
  #[tracing::instrument(skip(self), level = "TRACE")]
  pub fn apply_sql_filter(&self, user_id: &str, query: &str) -> Result<String> {
    #[cfg(feature = "disable-auth")]
    {
      return Ok(query.to_string());
    }
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
    |table_with_joins: &mut TableWithJoins| -> Result<(String, Ident)> {
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
            alias.as_ref().map(|a| a.name.clone()).unwrap(),
          ))
        }
        _ => Err(anyhow!("Unsupported query")),
      }
    };

  let build_selection_filter =
    |table: &str, alias: &Ident, acl_type: &AclType| {
      let table_acls =
        acls_by_table.get(table).and_then(|acls| acls.get(acl_type));
      let wildcard_table_acls =
        acls_by_table.get("*").and_then(|acls| acls.get(acl_type));

      if table_acls.is_none() && wildcard_table_acls.is_none() {
        bail!(format!(
          "Doesn't have {:?} access [table = {}]",
          acl_type, table
        ));
      }

      // Note: wildcard tables shouldn't have filters, so dont use it here
      let mut filters: Option<Expr> = table_acls.and_then(|exprs| {
        if exprs.is_empty() {
          None
        } else {
          exprs
            .iter()
            .map(|expr| expr.to_owned())
            .reduce(|agg, expr| Expr::BinaryOp {
              left: Box::new(agg),
              op: BinaryOperator::Or,
              right: Box::new(expr),
            })
            .map(|filters| Expr::Nested(Box::new(filters)))
        }
      });

      if let Some(filters) = filters.as_mut() {
        visit_expressions_mut(filters, |expr| {
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
              .collect::<Result<BTreeMap<String, Ident>>>();
            let table_alias = match table_alias {
              Ok(alias) => alias,
              Err(e) => {
                err = Some(e);
                return ControlFlow::Break(());
              }
            };

            visit_expressions_mut(select, |expr| {
              match expr {
                Expr::CompoundIdentifier(ids) => {
                  // if compound identifier is used, for eg: "table"."id",
                  // change the "table" to the ident of it's alias
                  if ids.len() == 2 {
                    if let Some(alias) = table_alias.get(&ids[0].value) {
                      ids[0] = alias.clone();
                    }
                  } else if ids.len() == 3 {
                    if let Some(alias) = table_alias.get(&ids[1].value) {
                      ids[1] = alias.clone();
                    }
                  }
                }
                _ => {}
              }
              ControlFlow::<()>::Continue(())
            });

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
          && !acls_by_table
            .get("*")
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

        // TODO: figure out alias replacement for assignment, returning, etc
        visit_expressions_mut(selection, |expr| {
          match expr {
            Expr::CompoundIdentifier(ids) => {
              // if compound identifier is used, for eg: "table"."id",
              // change the "table" to the ident of it's alias
              if ids.len() == 2 {
                if ids[0].value == table_alias.0 {
                  ids[0] = table_alias.1.clone();
                }
              } else if ids.len() == 3 {
                if ids[1].value == table_alias.0 {
                  ids[1] = table_alias.1.clone();
                }
              }
            }
            _ => {}
          }
          ControlFlow::<()>::Continue(())
        });

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
          .collect::<Result<BTreeMap<String, Ident>>>();

        let table_alias = match table_alias {
          Ok(alias) => alias,
          Err(e) => {
            err = Some(e);
            return ControlFlow::Break(());
          }
        };

        visit_expressions_mut(selection, |expr| {
          match expr {
            Expr::CompoundIdentifier(ids) => {
              // if compound identifier is used, for eg: "table"."id",
              // change the "table" to the ident of it's alias
              if ids.len() == 2 {
                if let Some(alias) = table_alias.get(&ids[0].value) {
                  ids[0] = alias.clone();
                }
              } else if ids.len() == 3 {
                if let Some(alias) = table_alias.get(&ids[1].value) {
                  ids[1] = alias.clone();
                }
              }
            }
            _ => {}
          }
          ControlFlow::<()>::Continue(())
        });

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

  // itertools group_by doesnt group accurately if the vec isn't sorted
  // so, test that vec with non-sorted users and tables are grouped
  // properly
  #[test]
  fn test_rowacl_set_acls_for_multiple_tables_random_order() {
    let mut checker = RowAclChecker::from(vec![]).unwrap();
    checker.set_acls(vec![
      acl("user_1", "table_2", AclType::Select, "*"),
      acl("user_1", "table_1", AclType::Select, "id = 1"),
      acl("user_2", "table_3", AclType::Select, "*"),
      acl("user_1", "table_1", AclType::Update, "*"),
    ]);
    assert_eq!(checker.acls_by_user_id.get("user_1").unwrap().len(), 2);
    assert_eq!(checker.acls_by_user_id.get("user_2").unwrap().len(), 1);
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
  fn test_rowacl_check_query_access_on_wildcard_table() {
    let checker = RowAclChecker::from(vec![
      acl("user_1", "*", AclType::Select, "*"),
      acl("user_1", "*", AclType::Insert, "*"),
      acl("user_1", "*", AclType::Update, "*"),
      acl("user_1", "*", AclType::Delete, "*"),
    ])
    .unwrap();
    assert!(checker.has_query_access("user_1", "table_2", AclType::Select));
    assert!(checker.has_query_access("user_1", "table_2", AclType::Insert));
    assert!(checker.has_query_access("user_1", "table_2", AclType::Update));
    assert!(checker.has_query_access("user_1", "table_2", AclType::Delete));

    // no filter applied for wildcard table access with wildcard filter
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1")
        .unwrap(),
      "SELECT * FROM table1 AS t1".to_owned()
    );
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "INSERT INTO table1 VALUES(1)")
        .unwrap(),
      "INSERT INTO table1 VALUES (1)".to_owned()
    );
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "UPDATE table1 SET id = 1")
        .unwrap(),
      "UPDATE table1 AS t1 SET id = 1".to_owned()
    );
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "DELETE FROM table1")
        .unwrap(),
      "DELETE FROM table1 AS t1".to_owned()
    );
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
      "SELECT * FROM table1 AS t1 WHERE (t1.id = 1)".to_owned()
    );

    // with alias, should preserve alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1 t")
        .unwrap(),
      "SELECT * FROM table1 AS t WHERE (t.id = 1)".to_owned()
    );

    // with quoted table name
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM \"table1\" t")
        .unwrap(),
      "SELECT * FROM \"table1\" AS t WHERE (t.id = 1)".to_owned()
    );

    // with quoted alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM \"table1\" \"t\"")
        .unwrap(),
      "SELECT * FROM \"table1\" AS \"t\" WHERE (\"t\".id = 1)".to_owned()
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
      "SELECT * FROM table1 AS t1 WHERE (t1.id = 1 AND t1.age < 10)".to_owned()
    );
  }

  #[test]
  fn test_rowacl_apply_multiple_acls_with_conditional_select() {
    let checker = RowAclChecker::from(vec![
      acl("user_1", "table1", AclType::Select, "id = 2"),
      acl("user_1", "table1", AclType::Select, "id = 10"),
    ])
    .unwrap();

    // without alias, should add alias
    assert_eq!(
      checker
        .apply_sql_filter("user_1", "SELECT * FROM table1 WHERE id > 1")
        .unwrap(),
      "SELECT * FROM table1 AS t1 WHERE id > 1 AND (t1.id = 2 OR t1.id = 10)"
        .to_owned()
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
  fn test_rowacl_apply_select_filter_on_query_using_table_reference() {
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
        .apply_sql_filter(
          "user_1",
          "SELECT * FROM table1 WHERE \"table1\".\"id\" > 10 AND \"table1\".\"age\" = 99"
        )
        .unwrap().as_str(),
      "SELECT * FROM table1 AS t1 WHERE t1.\"id\" > 10 AND t1.\"age\" = 99 AND (t1.id = 1)"
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
      "SELECT * FROM table1 AS t1 WHERE (t1.id = 1 OR t1.id = 2)".to_owned()
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
        "SELECT * FROM table1 AS t1 WHERE (t1.name ILIKE 'arena%' OR t1.age > 10 OR t1.id IN (69, 420))".to_owned()
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
      "UPDATE table1 AS t1 SET name = 'my name' WHERE id = 1111 AND (t1.id = 1)"
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
        "DELETE FROM table1 AS t1 WHERE name = 'my name' AND id = 1111 AND (t1.id = 99)"
          .to_owned()
    );
  }
}
