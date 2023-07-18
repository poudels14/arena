pub mod collections;
pub mod contents;
pub mod documents;
pub mod embeddings;

pub use collections::{Collection, CollectionsHandle};
pub use contents::DocumentContentsHandle;
pub use documents::{Document, DocumentsHandle};
pub use embeddings::DocumentEmbeddingsHandle;
