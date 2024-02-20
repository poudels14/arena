mod array;
mod cell;
mod column;
mod constraint;
mod dataframe;
mod datatype;
mod index;
mod row;
mod table;

pub(self) mod proto {
  include!(concat!(env!("OUT_DIR"), "/arenasql.schema.rs"));
  pub use self::table_index::Provider as TableIndexProvider;
}

pub use array::ColumnArrayBuilder;
pub use cell::{OwnedSerializedCell, SerializedCell};
pub use column::{Column, ColumnId, ColumnProperty, CTID_COLUMN};
pub use constraint::Constraint;
pub use dataframe::DataFrame;
pub use datatype::DataType;
pub use index::{IndexProvider, TableIndex, TableIndexId, VectorMetric};
pub use row::{OwnedRow, Row, RowId, RowTrait};
pub use table::{Table, TableId};
