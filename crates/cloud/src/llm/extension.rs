use crate::llm::tokenizer::{
  op_cloud_llm_hf_encode, op_cloud_llm_hf_new_pretrained_tokenizer,
};
use common::deno::extensions::BuiltinExtension;
use deno_core::Extension;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("arena/cloud/llm")
    .ops(vec![
      op_cloud_llm_hf_new_pretrained_tokenizer::decl(),
      op_cloud_llm_hf_encode::decl(),
    ])
    .force_op_registration()
    .build()
}
