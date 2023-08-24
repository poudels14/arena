use anyhow::Result;
use vectordb::query::{Collection, Document};
use vectordb::{DatabaseOptions, VectorDatabase};

pub fn main() -> Result<()> {
  let path = "test-db";
  let mut db = VectorDatabase::open(
    path,
    DatabaseOptions {
      enable_statistics: true,
    },
  )?;

  let collection_id = "test".into();
  db.create_collection(
    collection_id,
    Collection {
      dimension: 4,

      ..Default::default()
    },
  )?;

  let doc_id = "doc-1".into();
  db.add_document(
    collection_id,
    doc_id,
    Document {
      blobs: vec![("raw".to_string(), "raw-content".as_bytes().to_vec())],
      ..Default::default()
    },
  )?;

  let blobs =
    db.get_document_blobs(collection_id, doc_id, vec!["raw".to_owned()])?;
  println!(
    "Blobs = {:?}",
    blobs
      .iter()
      .map(|(k, v)| {
        (
          k.to_owned(),
          v.as_ref()
            .map(|v| std::str::from_utf8(&v).unwrap().to_owned())
            .unwrap(),
        )
      })
      .collect::<Vec<(String, String)>>()
  );

  let collection = db.get_collection(collection_id)?.unwrap();
  println!("Collection blobs = {:?}", collection.blobs);

  db.close()?;
  drop(db);

  VectorDatabase::destroy(path)?;
  Ok(())
}
