use super::SearchOptions;
use crate::db::rocks::cf::DatabaseColumnFamily;
use crate::db::storage::embeddings::StoredEmbeddings;
use crate::db::storage::{self};
use crate::db::{lock_error, VectorDatabase};
use crate::storage::row::RowId;
use crate::vectors::scoring::sortedscore::SortedSimilarityScores;
use crate::vectors::scoring::{SimilarityScorerFactory, SimilarityType};
use crate::vectors::VectorElement;
use anyhow::{bail, Result};
use bitvec::field::BitField;
use bitvec::prelude::Msb0;
use bitvec::view::BitView;
use bstr::BStr;
use indexmap::IndexMap;
use rocksdb::ReadOptions;
use serde_json::Value;

pub struct FsSearch<'a> {
  db: &'a VectorDatabase,
}

impl<'a> FsSearch<'a> {
  pub fn using(db: &'a VectorDatabase) -> Self {
    Self { db }
  }

  #[allow(dead_code)]
  pub fn top_k(
    &self,
    collection_id: &BStr,
    query: &[VectorElement],
    k: usize,
    options: SearchOptions,
  ) -> Result<Vec<(f32, (RowId, u32, u32, u32, IndexMap<String, Value>))>> {
    let collection = self.db.get_internal_collection(collection_id)?;
    let collection = collection.lock().map_err(lock_error)?;

    let query_len = query.len();
    if query_len != collection.dimension as usize {
      bail!("Query vector dimension not same as document embedding dimension")
    } else if query_len % 4 != 0 {
      bail!("Query vector dimension should be a multiple of 4")
    }

    let embeddings_cf = storage::embeddings::cf(&self.db.db)?;
    let mut read_options = ReadOptions::default();
    read_options.fill_cache(false);
    read_options.set_iterate_upper_bound((collection.index + 1).to_be_bytes());

    let scorer = SimilarityScorerFactory::get_default(SimilarityType::Dot);
    let mut scores = SortedSimilarityScores::new(k);
    let min_score = options.min_score.unwrap_or(0.0);

    embeddings_cf
      .prefix_iterator_opt(&collection.index.to_be_bytes(), read_options)
      .filter_map(|embedding| embedding.ok())
      .for_each(|(key, embedding)| {
        let embedding =
          unsafe { rkyv::archived_root::<StoredEmbeddings>(&embedding) };
        let score = scorer.similarity(&query, &embedding.vectors);

        if score >= min_score {
          // Note(sagar): decode these as i32 since they are stored as i32
          let doc_index: u32 = key[4..8].view_bits::<Msb0>().load_be();
          let chunk_index: u32 = key[8..12].view_bits::<Msb0>().load_be();
          scores.push((
            score,
            (
              doc_index,
              chunk_index,
              embedding.start,
              embedding.end,
              embedding.metadata.to_vec(),
            ),
          ));
        }
      });

    scores
      .as_vec()
      .iter()
      .map(|(score, info)| {
        Ok((
          score.to_owned(),
          (
            RowId {
              collection_index: collection.index,
              row_index: info.0,
            },
            info.1,
            info.2,
            info.3,
            rmp_serde::from_slice::<IndexMap<String, Value>>(&info.4)?,
          ),
        ))
      })
      .collect()
  }
}
