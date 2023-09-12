pub mod collections;
pub mod contents;
pub mod documents;
pub mod embeddings;
pub mod row;

pub use collections::{Collection, CollectionsHandle};
pub use contents::DocumentBlobsHandle;
pub use documents::{Document, DocumentsHandle};
pub use embeddings::DocumentEmbeddingsHandle;
use rocksdb::{OptimisticTransactionDB, SingleThreaded};

pub type Database = OptimisticTransactionDB<SingleThreaded>;
