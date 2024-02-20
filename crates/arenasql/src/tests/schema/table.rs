use crate::schema::{
  Column, ColumnProperty, Constraint, DataType, IndexProvider, Table,
  TableIndex, VectorMetric,
};
use crate::tests::create_session_context;

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_shouldnt_be_found_after_its_dropped() {
  let session = create_session_context();

  let _ = session
    .execute_sql(
      r#"CREATE TABLE IF NOT EXISTS unique_column (
        id VARCHAR(50),
        name TEXT
      )"#,
    )
    .await
    .unwrap();

  let res = session.execute_sql(r#"DROP TABLE unique_column"#).await;
  assert!(res.is_ok());

  let res = session.execute_sql(r#"SELECT * FROM unique_column;"#).await;
  assert!(res.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn table_schema_protobuf_encoding_decoding() {
  let table = Table {
    id: 2,
    name: "test_table".to_owned(),
    columns: vec![Column {
      id: 8,
      name: "column_1".to_owned(),
      data_type: DataType::Jsonb,
      properties: ColumnProperty::NOT_NULL,
      default_value: None,
    }],
    constraints: vec![Constraint::Unique(vec![1])],
    indexes: vec![
      TableIndex {
        id: 12,
        name: "index_1".to_owned(),
        provider: IndexProvider::BasicIndex {
          columns: vec![0],
          unique: false,
        },
      },
      TableIndex {
        id: 12,
        name: "index_2".to_owned(),
        provider: IndexProvider::HNSWIndex {
          columns: vec![1],
          metric: VectorMetric::Dot,
          m: 16,
          ef_construction: 16,
          ef: 16,
          dim: 10,
          retain_vectors: false,
          namespace_column: Some(1),
        },
      },
    ],
  };

  let proto = table.to_protobuf().unwrap();

  let decoded = Table::from_protobuf(&proto).unwrap();
  assert_eq!(table, decoded);
}
