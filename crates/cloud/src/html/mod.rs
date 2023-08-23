pub mod extractor;

use crate::html::extractor::TextExtractor;
use anyhow::bail;
use anyhow::Result;
use common::deno::extensions::BuiltinExtension;
use deno_core::op;
use deno_core::Extension;
use deno_core::OpState;
use deno_core::StringOrBuffer;
use html5ever::tendril::SliceExt;
use html5ever::tendril::Tendril;
use html5ever::tokenizer::{
  BufferQueue, Tokenizer, TokenizerOpts, TokenizerResult,
};
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;

pub fn extension() -> BuiltinExtension {
  BuiltinExtension {
    extension: Some(self::init()),
    runtime_modules: vec![],
    snapshot_modules: vec![],
  }
}

pub(crate) fn init() -> Extension {
  Extension::builder("arena/cloud/html")
    .ops(vec![op_cloud_html_extract_text::decl()])
    .force_op_registration()
    .build()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractTextOptions {
  /// Tags to ignore when extracting text; for example, "style", "script", etc
  #[serde(default)]
  ignore_tags: Vec<String>,

  /// Ignore text node if it's whitespaces only
  /// For example, newlines between tags will be
  /// included if this isn't set to false
  #[serde(default)]
  skip_whitespaces: bool,
}

#[op]
async fn op_cloud_html_extract_text(
  _state: Rc<RefCell<OpState>>,
  html: StringOrBuffer,
  options: ExtractTextOptions,
) -> Result<Vec<String>> {
  let extractor = TextExtractor::new(Default::default());

  let chunk: Tendril<_> = html.to_tendril();
  let mut input = BufferQueue::new();
  input.push_back(chunk.try_reinterpret().unwrap());

  let mut tok = Tokenizer::new(
    extractor,
    TokenizerOpts {
      profile: false,
      ..Default::default()
    },
  );
  let res = tok.feed(&mut input);
  if let TokenizerResult::Script(h) = res {
    bail!("Parsig failed = {:?}", h);
  }

  let texts = tok.sink.get_texts();
  let filtered_texts = texts
    .iter()
    .filter_map(|t| {
      if options.ignore_tags.contains(&t.0) {
        return None;
      }
      if options.skip_whitespaces && t.1.trim().len() == 0 {
        return None;
      }
      return Some(t.1.clone());
    })
    .collect();

  Ok(filtered_texts)
}
