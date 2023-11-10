use common::deno::extensions::{BuiltinExtension, BuiltinExtensionProvider};
use common::resolve_from_root;
use deno_core::{Extension, Op};

use crate::jwt::{op_cloud_jwt_sign, op_cloud_jwt_verify};
use crate::pubsub::publisher::Publisher;
use crate::transpile::op_cloud_transpile_js_data_query;
use crate::{html, llm, pdf, pubsub, vectordb};

macro_rules! cloud_module {
  ($module:literal) => {{
    (
      concat!("@arena/cloud/", $module),
      resolve_from_root!(concat!(
        "../../js/arena-runtime/dist/cloud/",
        $module,
        ".js"
      )),
    )
  }};
}

#[derive(Default, Clone)]
pub struct Config {
  pub publisher: Option<Publisher>,
}

pub struct CloudExtensionProvider {
  pub publisher: Option<Publisher>,
}

impl BuiltinExtensionProvider for CloudExtensionProvider {
  fn get_extension(&self) -> BuiltinExtension {
    extension(Config {
      publisher: self.publisher.clone(),
    })
  }
}

pub fn extension(options: Config) -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init(options)),
    runtime_modules: vec![],
    snapshot_modules: vec![
      // TODO(sagar): load these during snapshotting
      cloud_module!("jwt"),
      cloud_module!("pubsub"),
      cloud_module!("query"),
      cloud_module!("llm"),
      cloud_module!("vectordb"),
      cloud_module!("pdf"),
      cloud_module!("html"),
    ],
  }
}

pub(crate) fn init(options: Config) -> Extension {
  Extension {
    name: "arena/cloud",
    ops: vec![
      // pubsub
      pubsub::extension::op_cloud_pubsub_publish::DECL,
      pubsub::extension::op_cloud_pubsub_subscribe::DECL,
      // data query transpiler
      op_cloud_transpile_js_data_query::DECL,
      // jwt
      op_cloud_jwt_sign::DECL,
      op_cloud_jwt_verify::DECL,
      // llm
      llm::tokenizer::op_cloud_llm_hf_new_pretrained_tokenizer::DECL,
      llm::tokenizer::op_cloud_llm_hf_encode::DECL,
      // pdf
      pdf::html::op_cloud_pdf_to_html::DECL,
      // html
      html::op_cloud_html_extract_text::DECL,
      // vector db
      vectordb::op_cloud_vectordb_open::DECL,
      vectordb::op_cloud_vectordb_execute_query::DECL,
      vectordb::op_cloud_vectordb_create_collection::DECL,
      vectordb::op_cloud_vectordb_list_collections::DECL,
      vectordb::op_cloud_vectordb_get_collection::DECL,
      vectordb::op_cloud_vectordb_add_document::DECL,
      vectordb::op_cloud_vectordb_list_documents::DECL,
      vectordb::op_cloud_vectordb_get_document::DECL,
      vectordb::op_cloud_vectordb_get_document_blobs::DECL,
      vectordb::op_cloud_vectordb_set_document_embeddings::DECL,
      vectordb::op_cloud_vectordb_delete_document::DECL,
      vectordb::op_cloud_vectordb_search_collection::DECL,
      vectordb::op_cloud_vectordb_compact_and_flush::DECL,
    ]
    .into(),
    op_state_fn: Some(Box::new(|state| {
      if let Some(publisher) = options.publisher {
        state.put::<Publisher>(publisher);
      }
    })),
    enabled: true,
    ..Default::default()
  }
}
