use common::deno::extensions::BuiltinExtension;
use common::resolve_from_root;
use deno_core::Extension;

use crate::jwt::{op_cloud_jwt_sign, op_cloud_jwt_verify};
use crate::transpile::op_cloud_transpile_js_data_query;
use crate::{html, llm, pdf, vectordb};

macro_rules! cloud_module {
  ($module:literal) => {{
    (
      concat!("@arena/cloud/", $module),
      resolve_from_root!(
        concat!("../../js/arena-runtime/dist/cloud/", $module, ".js"),
        true
      ),
    )
  }};
}

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![
      // TODO(sagar): load these during snapshotting
      cloud_module!("jwt"),
      cloud_module!("query"),
      cloud_module!("llm"),
      cloud_module!("vectordb"),
      cloud_module!("pdf"),
      cloud_module!("html"),
    ],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("arena/cloud")
    .ops(vec![
      op_cloud_transpile_js_data_query::decl(),
      // jwt
      op_cloud_jwt_sign::decl(),
      op_cloud_jwt_verify::decl(),
      // llm
      llm::tokenizer::op_cloud_llm_hf_new_pretrained_tokenizer::decl(),
      llm::tokenizer::op_cloud_llm_hf_encode::decl(),
      // pdf
      pdf::html::op_cloud_pdf_to_html::decl(),
      // html
      html::op_cloud_html_extract_text::decl(),
      // vector db
      vectordb::op_cloud_vectordb_open::decl(),
      vectordb::op_cloud_vectordb_execute_query::decl(),
      vectordb::op_cloud_vectordb_create_collection::decl(),
      vectordb::op_cloud_vectordb_list_collections::decl(),
      vectordb::op_cloud_vectordb_get_collection::decl(),
      vectordb::op_cloud_vectordb_add_document::decl(),
      vectordb::op_cloud_vectordb_list_documents::decl(),
      vectordb::op_cloud_vectordb_get_document::decl(),
      vectordb::op_cloud_vectordb_get_document_blobs::decl(),
      vectordb::op_cloud_vectordb_set_document_embeddings::decl(),
      vectordb::op_cloud_vectordb_delete_document::decl(),
      vectordb::op_cloud_vectordb_search_collection::decl(),
      vectordb::op_cloud_vectordb_compact_and_flush::decl(),
    ])
    .force_op_registration()
    .build()
}
