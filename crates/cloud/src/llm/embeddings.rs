use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use anyhow::{anyhow, Error, Result};
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, HiddenAct, DTYPE};
use deno_core::{op2, OpState, Resource, ResourceId};
use hf_hub::api::sync::Api;
use hf_hub::{Repo, RepoType};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingsModelOptions {
  model_id: Option<String>,
  revision: Option<String>,
  #[serde(default)]
  use_pth: bool,
  #[serde(default)]
  approximate_gelu: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEmbeddingsOptions {
  #[serde(default)]
  normalize: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenizeOptions {
  truncate: Option<bool>,
  max_length: Option<usize>,
}

pub struct EmbeddingsModel {
  tokenizer: Tokenizer,
  model: BertModel,
}

impl Resource for EmbeddingsModel {}

// Credit: hugging face
#[op2]
#[smi]
pub fn op_cloud_llm_embeddings_load_model(
  state: Rc<RefCell<OpState>>,
  #[serde] options: EmbeddingsModelOptions,
) -> Result<ResourceId> {
  let device = Device::Cpu;
  let default_model = "sentence-transformers/all-MiniLM-L6-v2".to_string();
  let default_revision = "refs/pr/21".to_string();
  let (model_id, revision) =
    match (options.model_id.to_owned(), options.revision.to_owned()) {
      (Some(model_id), Some(revision)) => (model_id, revision),
      (Some(model_id), None) => (model_id, "main".to_string()),
      (None, Some(revision)) => (default_model, revision),
      (None, None) => (default_model, default_revision),
    };

  tracing::debug!("Loading model: {}", model_id);
  let start = Instant::now();
  let repo = Repo::with_revision(model_id, RepoType::Model, revision);
  let (config_filename, tokenizer_filename, weights_filename) = {
    let api = Api::new()?;
    let api = api.repo(repo);
    let config = api.get("config.json")?;
    let tokenizer = api.get("tokenizer.json")?;
    let weights = if options.use_pth {
      api.get("pytorch_model.bin")?
    } else {
      api.get("model.safetensors")?
    };
    (config, tokenizer, weights)
  };
  let config = std::fs::read_to_string(config_filename)?;
  let mut config: Config = serde_json::from_str(&config)?;

  let vb = if options.use_pth {
    VarBuilder::from_pth(&weights_filename, DTYPE, &device)?
  } else {
    unsafe {
      VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)?
    }
  };
  if options.approximate_gelu {
    config.hidden_act = HiddenAct::GeluApproximate;
  }

  let tokenizer =
    Tokenizer::from_file(tokenizer_filename).map_err(anyhow::Error::msg)?;

  let model = BertModel::load(vb, &config)?;
  tracing::debug!("Model loaded. Time taken = {}", start.elapsed().as_millis());

  Ok(
    state
      .borrow_mut()
      .resource_table
      .add(EmbeddingsModel { tokenizer, model }),
  )
}

// Credit: hugging face
#[op2(async)]
#[serde]
pub async fn op_cloud_llm_embeddings_generate(
  state: Rc<RefCell<OpState>>,
  #[smi] resource_id: ResourceId,
  #[serde] texts: Vec<String>,
  #[serde] options: GenerateEmbeddingsOptions,
) -> Result<serde_json::Value> {
  let model = state
    .borrow()
    .resource_table
    .get::<EmbeddingsModel>(resource_id)?
    .clone();

  let mut tokenizer = model.tokenizer.clone();
  if let Some(pp) = tokenizer.get_padding_mut() {
    pp.strategy = PaddingStrategy::BatchLongest
  } else {
    let pp = PaddingParams {
      strategy: PaddingStrategy::BatchLongest,
      ..Default::default()
    };
    tokenizer.with_padding(Some(pp));
  }

  let device = Device::Cpu;
  let tokens = tokenizer.encode_batch(texts, true).map_err(Error::msg)?;
  let token_ids = tokens
    .iter()
    .map(|tokens| {
      let tokens = tokens.get_ids().to_vec();
      Ok(Tensor::new(tokens.as_slice(), &device)?)
    })
    .collect::<Result<Vec<_>>>()?;
  let token_ids = Tensor::stack(&token_ids, 0)?;
  let token_type_ids = token_ids.zeros_like()?;
  let embeddings = model.model.forward(&token_ids, &token_type_ids)?;

  let (_n_sentence, n_tokens, _hidden_size) = embeddings.dims3()?;
  let embeddings = (embeddings.sum(1)? / (n_tokens as f64))?;
  let embeddings = match options.normalize {
    true => normalize_l2(&embeddings)?,
    false => embeddings,
  };

  Ok(json!(embeddings.to_vec2::<f32>().unwrap()))
}

#[op2(async)]
#[serde]
pub async fn op_cloud_llm_embeddings_tokenize(
  state: Rc<RefCell<OpState>>,
  #[smi] resource_id: ResourceId,
  #[string] text: String,
  #[serde] options: TokenizeOptions,
) -> Result<serde_json::Value> {
  let model = state
    .borrow()
    .resource_table
    .get::<EmbeddingsModel>(resource_id)?
    .clone();

  let mut tokenizer = model.tokenizer.clone();
  tokenizer.with_padding(None);
  if let Some(max_length) = options.max_length {
    tokenizer
      .with_truncation(Some(TruncationParams {
        max_length,
        ..Default::default()
      }))
      .unwrap();
  }
  match options.truncate {
    Some(truncate) if !truncate => {
      tokenizer.with_truncation(None).unwrap();
    }
    _ => {}
  }

  let encoding = tokenizer
    .encode(text, false)
    .map_err(|e| anyhow!("{:?}", e))?;

  Ok(json!({
    "ids": encoding.get_ids(),
    "offsetMapping": encoding.get_offsets()
  }))
}

#[op2]
#[serde]
pub fn op_cloud_llm_embeddings_close_model(
  state: Rc<RefCell<OpState>>,
  #[smi] resource_id: ResourceId,
) -> Result<()> {
  state
    .borrow_mut()
    .resource_table
    .take::<EmbeddingsModel>(resource_id)?;
  Ok(())
}

pub fn normalize_l2(v: &Tensor) -> Result<Tensor> {
  Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}
