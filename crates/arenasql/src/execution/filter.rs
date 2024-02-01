use std::cmp::Ordering;

use datafusion::logical_expr::{Expr, Like, Operator};

use crate::schema::{OwnedSerializedCell, Table, TableIndex};
use crate::{Error, Result};

#[derive(Debug, Clone)]
pub enum Filter {
  BinaryExpr {
    projected_columns: Vec<usize>,
    left: Box<Expr>,
    op: Operator,
    right: Box<Expr>,
  },
  IsNotNull {
    projected_columns: Vec<usize>,
    expr: Box<Expr>,
  },
  Like {
    projected_columns: Vec<usize>,
    expr: Like,
  },
  IsNull {
    projected_columns: Vec<usize>,
    expr: Box<Expr>,
  },
  IsTrue {
    projected_columns: Vec<usize>,
    expr: Box<Expr>,
  },
  IsFalse {
    projected_columns: Vec<usize>,
    expr: Box<Expr>,
  },
}

impl Filter {
  pub fn for_table(table: &Table, expr: &Expr) -> Result<Self> {
    let filter_cols: Vec<String> =
      expr.to_columns()?.iter().map(|c| c.name.clone()).collect();

    // ordered projected columns
    let projected_columns: Vec<usize> = table
      .columns
      .iter()
      .enumerate()
      .filter(|tcol| filter_cols.contains(&tcol.1.name))
      .map(|p| p.0)
      .collect();

    // TODO: support more than 1 projected column?
    if projected_columns.len() > 1 {
      return Err(Error::UnsupportedOperation(format!(
        "Unsupported filter: {}",
        expr.to_string()
      )));
    }

    match expr {
      Expr::BinaryExpr(e) => Ok(Self::BinaryExpr {
        projected_columns,
        left: e.left.clone(),
        op: e.op.clone(),
        right: e.right.clone(),
      }),
      Expr::IsNotNull(e) => Ok(Self::IsNotNull {
        projected_columns,
        expr: e.clone(),
      }),
      Expr::Like(e) => Ok(Self::Like {
        projected_columns,
        expr: e.clone(),
      }),
      Expr::IsNull(e) => Ok(Self::IsNull {
        projected_columns,
        expr: e.clone(),
      }),
      Expr::IsTrue(e) => Ok(Self::IsTrue {
        projected_columns,
        expr: e.clone(),
      }),
      Expr::IsFalse(e) => Ok(Self::IsFalse {
        projected_columns,
        expr: e.clone(),
      }),
      _ => Err(Error::UnsupportedQueryFilter(expr.to_string())),
    }
  }

  #[inline]
  pub fn get_column_projection(&self) -> &Vec<usize> {
    match self {
      Self::BinaryExpr {
        projected_columns, ..
      }
      | Self::IsNotNull {
        projected_columns, ..
      }
      | Self::Like {
        projected_columns, ..
      }
      | Self::IsNull {
        projected_columns, ..
      }
      | Self::IsTrue {
        projected_columns, ..
      }
      | Self::IsFalse {
        projected_columns, ..
      } => projected_columns,
    }
  }

  /// Returns whether this filter does '=' comparision
  pub fn is_eq(&self) -> bool {
    match self {
      Self::BinaryExpr { op, .. } => match op {
        Operator::Eq => true,
        _ => false,
      },
      _ => false,
    }
  }

  /// Returns the literal used in '=' expression if the
  /// filter is of type '=', else returns `None`.
  pub fn get_binary_eq_literal<'a>(&'a self) -> Option<OwnedSerializedCell> {
    match self {
      Self::BinaryExpr {
        op, left, right, ..
      } => match op {
        Operator::Eq => match right.as_ref() {
          Expr::Literal(lit) => Some(OwnedSerializedCell::from_scalar(lit)),
          _ => match left.as_ref() {
            Expr::Literal(lit) => Some(OwnedSerializedCell::from_scalar(lit)),
            _ => None,
          },
        },
        _ => None,
      },
      _ => None,
    }
  }

  pub fn is_supported_by_index(&self, index: &TableIndex) -> bool {
    self
      .get_column_projection()
      .iter()
      .zip(index.columns().iter())
      .fold(true, |agg, (filter_col, index_col)| {
        agg && filter_col == index_col
      })
  }

  /// This is used to keep track of whether the filter will be properly
  /// applied to the rows during scanning such that datafusion doesn't
  /// have to re-apply the filter
  pub fn is_filter_pushdown_suported(&self) -> bool {
    match self {
      Self::BinaryExpr { op, .. } => match op {
        Operator::Eq => true,
        _ => false,
      },
      _ => false,
    }
  }

  #[inline]
  pub fn get_operator_cost(&self) -> f32 {
    return 0.0025;
  }

  /// Cost is calcualted in the following way
  /// n = total number of rows
  /// m = average rows per unique column value
  ///
  /// unique index with exact columns:
  /// cost = row_filter_cost
  ///
  /// secondary index:
  /// first column cost = n/m * row_filter_cost
  /// second column cost = n/2m * row_filter_cost
  /// third column cost = n/3m * row_filter_cost
  ///
  /// if first column doesn't match,
  /// cost = n [because of entire index scan] * row_filter_cost
  pub fn estimate_cost(&self, index: &TableIndex) -> f32 {
    let index_columns = index.columns();
    let matched_cols = self
      .get_column_projection()
      .iter()
      .zip(index_columns.iter())
      .take_while(|(filter_col, index_col)| filter_col == index_col)
      .count();

    // If the filter is '=', it doesn't require index scan,
    // so, the cost is O(1)
    if matched_cols == index_columns.len() && self.is_eq() {
      return self.get_operator_cost() * 1.0;
    }
    // TODO: penalize the index if the index doesn't have all the
    // columns used in the filter

    // TODO: use estimated row count instead of 10_000
    self.get_operator_cost() * 10_000.0 / matched_cols as f32
  }

  /// Returns the minimum cost of using the filters on the given index
  pub fn find_index_with_lowest_cost<'a>(
    indexes: &'a Vec<TableIndex>,
    filters: &'a Vec<Filter>,
  ) -> Option<&'a TableIndex> {
    if filters.is_empty() {
      return None;
    }
    indexes
      .iter()
      .map(|index| {
        let lowest_cost = filters
          .iter()
          .map(|filter| filter.estimate_cost(index))
          .min_by(|a, b| {
            if a > b {
              Ordering::Greater
            } else {
              Ordering::Less
            }
          })
          .unwrap_or(f32::INFINITY);
        (index, lowest_cost)
      })
      .min_by(|index1, index2| {
        if index1.1 > index2.1 {
          Ordering::Greater
        } else {
          Ordering::Less
        }
      })
      .map(|(index, _)| index)
  }
}
