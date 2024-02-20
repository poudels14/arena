use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use datafusion::arrow::datatypes::{
  DataType as DfDataType, Field as DfField, Schema as DfSchema,
  SchemaRef as DfSchemaRef,
};
use datafusion::common::Constraints as DfConstraints;
use datafusion::datasource::TableProvider as DfTableProvider;
use inflector::Inflector;
use prost::Message;
use sqlparser::ast::{ColumnOption, Statement};

use super::column::CTID_COLUMN;
use super::index::IndexProvider;
use super::{
  Column, ColumnId, ColumnProperty, Constraint, DataType, OwnedSerializedCell,
  TableIndex, TableIndexId,
};
use crate::storage::Serializer;
use crate::Result;

pub type TableId = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
  pub id: TableId,
  pub name: String,
  pub columns: Vec<Column>,
  pub constraints: Vec<Constraint>,
  pub indexes: Vec<TableIndex>,
}

impl Table {
  pub fn from_provider(
    id: TableId,
    name: &str,
    provider: Arc<dyn DfTableProvider>,
    stmt: &Statement,
  ) -> Result<Self> {
    let columns = get_columns_from_query_stmt(stmt, provider.schema())?;
    let mut constraints: Vec<Constraint> = provider
      .constraints()
      .map(|constraints| {
        constraints
          .as_ref()
          .into_iter()
          .map(|c| Constraint::from(c))
          .collect()
      })
      .unwrap_or_default();

    // Add column constraint to table constraint
    columns.iter().for_each(|col| {
      if col.unique() {
        constraints.push(Constraint::Unique(vec![col.id as usize]));
      }
    });
    Ok(Table {
      id,
      name: name.to_owned(),
      columns,
      constraints,
      indexes: vec![],
    })
  }

  pub fn get_df_schema(&self) -> DfSchemaRef {
    let fields: Vec<DfField> = self
      .columns
      .iter()
      .map(|col| col.to_field(&self))
      .chain(vec![DfField::new(CTID_COLUMN, DfDataType::UInt64, false)
        .with_metadata(HashMap::from([(
          "TYPE".to_owned(),
          DataType::UInt64.to_string(),
        )]))])
      .collect();
    DfSchemaRef::new(DfSchema::new(fields))
  }

  pub fn get_df_constraints(&self) -> Option<DfConstraints> {
    if self.constraints.is_empty() {
      return None;
    }
    Some(DfConstraints::new_unverified(
      self.constraints.iter().map(|c| c.into()).collect(),
    ))
  }

  pub fn add_index(
    &mut self,
    index_id: TableIndexId,
    index_name: Option<String>,
    provider: IndexProvider,
  ) -> Result<TableIndex> {
    let index_name = index_name.unwrap_or_else(|| {
      let mut index_name = provider
        .columns()
        .iter()
        .fold(self.name.clone(), |agg, col| {
          agg + "_" + &self.columns[*col].name.to_snake_case()
        })
        + "_key";

      let index_name_overlap_count = self
        .indexes
        .iter()
        .filter(|idx| idx.name.starts_with(&index_name))
        .count();
      if index_name_overlap_count > 0 {
        index_name += &format!("_{}", index_name_overlap_count);
      }
      index_name
    });

    let index = TableIndex {
      id: index_id,
      name: index_name,
      provider,
    };
    self.indexes.push(index.clone());
    Ok(index)
  }

  pub fn project_columns(&self, projection: &[usize]) -> Vec<Column> {
    projection
      .iter()
      .map(|proj| self.columns[*proj].clone())
      .collect()
  }

  pub fn project_columns_names(&self, projection: &[usize]) -> Vec<&String> {
    projection
      .iter()
      .map(|proj| &self.columns[*proj].name)
      .collect()
  }

  pub fn from_protobuf(buf: &[u8]) -> Result<Self> {
    let table = super::proto::Table::decode(&mut Cursor::new(buf))?;
    Ok(Self {
      id: table.id as u16,
      name: table.name,
      columns: table
        .columns
        .iter()
        .map(|col| {
          Ok(Column {
            id: col.id as u8,
            name: col.name.clone(),
            data_type: Serializer::FixedInt
              .deserialize::<DataType>(&col.data_type)?,
            properties: ColumnProperty::from_bits(col.properties).unwrap(),
            default_value: col
              .default_value
              .as_ref()
              .map(|v| {
                Serializer::FixedInt.deserialize::<OwnedSerializedCell>(&v)
              })
              .transpose()?,
          })
        })
        .collect::<Result<Vec<Column>>>()?,
      constraints: table
        .constraints
        .iter()
        .map(|constraint| Constraint::from_proto(constraint))
        .collect(),
      indexes: table
        .indexes
        .iter()
        .map(|index| TableIndex::from_proto(index))
        .collect(),
    })
  }

  pub fn to_protobuf(&self) -> Result<Vec<u8>> {
    let table = super::proto::Table {
      id: self.id as u32,
      name: self.name.clone(),
      columns: self
        .columns
        .iter()
        .map(|col| {
          Ok(super::proto::Column {
            id: col.id as u32,
            name: col.name.clone(),
            data_type: Serializer::FixedInt
              .serialize::<DataType>(&col.data_type)?,
            properties: col.properties.bits(),
            default_value: col
              .default_value
              .as_ref()
              .map(|v| {
                Serializer::FixedInt.serialize::<OwnedSerializedCell>(&v)
              })
              .transpose()?,
          })
        })
        .collect::<Result<Vec<super::proto::Column>>>()?,
      constraints: self.constraints.iter().map(|c| c.to_proto()).collect(),
      indexes: self.indexes.iter().map(|index| index.to_proto()).collect(),
    };

    let mut buf = Vec::new();
    buf.reserve(table.encoded_len());
    table.encode(&mut buf)?;
    Ok(buf)
  }
}

fn get_columns_from_query_stmt(
  stmt: &Statement,
  schema: DfSchemaRef,
) -> Result<Vec<Column>> {
  match stmt {
    Statement::CreateTable { columns, .. } => columns
      .iter()
      .zip(schema.fields.iter())
      .enumerate()
      .map(|(index, (col, field))| {
        let mut properties = ColumnProperty::DEFAULT;

        if col
          .options
          .iter()
          .any(|opt| opt.option == ColumnOption::NotNull)
        {
          properties.insert(ColumnProperty::NOT_NULL);
        }

        if col.options.iter().any(|opt| {
          if let ColumnOption::Unique { .. } = opt.option {
            true
          } else {
            false
          }
        }) {
          properties.insert(ColumnProperty::UNIQUE);
        }

        Ok(Column {
          id: index as ColumnId,
          name: col.name.value.clone(),
          data_type: DataType::from_column_def(&col, Some(field.as_ref()))?,
          properties,
          default_value: None,
        })
      })
      .collect::<Result<Vec<Column>>>(),
    _ => unimplemented!(),
  }
}
