use std::sync::Arc;

use parking_lot::RwLock;
use runtime::deno::core::{Extension, Op};
use runtime::extensions::{BuiltinExtension, BuiltinExtensionProvider};

use crate::jwt::{op_cloud_jwt_sign, op_cloud_jwt_verify};
use crate::pubsub::publisher::Publisher;
use crate::rowacl::RowAclChecker;
use crate::transpile::op_cloud_transpile_js_data_query;
use crate::{html, llm, pdf, pubsub, rowacl, s3};

#[macro_export]
macro_rules! cloud_module {
  ($module:literal $(,)?) => {{
    // Preserve the code if "include-in-binary" feature is ON
    #[cfg(feature = "include-in-binary")]
    let source =
      runtime::extensions::SourceCode::Preserved(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../js/runtime/dist/cloud/",
        $module,
        ".js"
      )));

    // If the "include-in-binary" feature is off, dont need to include
    // the code unless "runtime" flag is ON, in which case, another macro
    // handles it
    #[cfg(not(feature = "include-in-binary"))]
    let source = runtime::extensions::SourceCode::NotPreserved;

    (concat!("@arena/cloud/", $module), source)
  }};
}

#[derive(Default, Clone)]
pub struct Config {
  pub publisher: Option<Publisher>,
  pub acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
}

#[derive(Default)]
pub struct CloudExtensionProvider {
  pub publisher: Option<Publisher>,
  pub acl_checker: Option<Arc<RwLock<RowAclChecker>>>,
}

impl BuiltinExtensionProvider for CloudExtensionProvider {
  fn get_extension(&self) -> BuiltinExtension {
    extension(Config {
      publisher: self.publisher.clone(),
      acl_checker: self.acl_checker.clone(),
    })
  }
}

pub fn extension(options: Config) -> BuiltinExtension {
  BuiltinExtension::new(
    Some(self::init(options)),
    vec![
      // TODO(sagar): load these during snapshotting
      cloud_module!("jwt"),
      cloud_module!("s3"),
      cloud_module!("pubsub"),
      cloud_module!("query"),
      cloud_module!("llm"),
      cloud_module!("pdf"),
      cloud_module!("html"),
    ],
  )
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
      // s3
      s3::op_cloud_s3_create_client::DECL,
      s3::op_cloud_s3_create_bucket::DECL,
      s3::op_cloud_s3_list_bucket::DECL,
      s3::op_cloud_s3_put_object::DECL,
      s3::op_cloud_s3_head_object::DECL,
      s3::op_cloud_s3_get_object::DECL,
      // rowacl
      rowacl::op_cloud_rowacl_new::DECL,
      rowacl::op_cloud_rowacl_has_access::DECL,
      rowacl::op_cloud_rowacl_apply_filters::DECL,
      rowacl::op_cloud_rowacl_close::DECL,
      rowacl::op_cloud_default_rowacl_apply_filters::DECL,
      // llm
      llm::tokenizer::op_cloud_llm_hf_new_pretrained_tokenizer::DECL,
      llm::tokenizer::op_cloud_llm_hf_encode::DECL,
      llm::embeddings::op_cloud_llm_embeddings_load_model::DECL,
      llm::embeddings::op_cloud_llm_embeddings_generate::DECL,
      llm::embeddings::op_cloud_llm_embeddings_tokenize::DECL,
      llm::embeddings::op_cloud_llm_embeddings_close_model::DECL,
      // pdf
      pdf::html::op_cloud_pdf_to_html::DECL,
      // html
      html::op_cloud_html_extract_text::DECL,
    ]
    .into(),
    op_state_fn: Some(Box::new(|state| {
      if let Some(publisher) = options.publisher {
        state.put::<Publisher>(publisher);
      }
      if let Some(checker) = options.acl_checker {
        state.put::<Arc<RwLock<RowAclChecker>>>(checker);
      }
    })),
    enabled: true,
    ..Default::default()
  }
}
