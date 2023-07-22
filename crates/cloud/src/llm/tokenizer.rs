use anyhow::{anyhow, bail, Result};
use deno_core::{op, OpState, Resource, ResourceId, StringOrBuffer};
use http::{HeaderMap, HeaderValue};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use tokenizers::{FromPretrainedParameters, Tokenizer};

struct TokenizerResource {
  tokenizer: Rc<Tokenizer>,
}

impl Resource for TokenizerResource {
  fn close(self: Rc<Self>) {
    drop(self)
  }
}

#[op]
async fn op_cloud_llm_hf_new_pretrained_tokenizer(
  state: Rc<RefCell<OpState>>,
  model_name: String,
) -> Result<ResourceId> {
  let mut tokenizer = Tokenizer::from_file(
    from_pretrained(
      model_name,
      Some(FromPretrainedParameters {
        ..Default::default()
      }),
    )
    .await?,
  )
  .map_err(|e| anyhow!("{:?}", e))?;
  tokenizer.with_padding(None);

  Ok(state.borrow_mut().resource_table.add(TokenizerResource {
    tokenizer: Rc::new(tokenizer),
  }))
}

#[op]
async fn op_cloud_llm_hf_encode<'a>(
  state: Rc<RefCell<OpState>>,
  tokenizer_id: ResourceId,
  text: StringOrBuffer,
) -> Result<Value> {
  let tokenizer = state
    .borrow()
    .resource_table
    .get::<TokenizerResource>(tokenizer_id)?
    .clone();

  let text_str = match text {
    StringOrBuffer::String(t) => t,
    StringOrBuffer::Buffer(buf) => {
      simdutf8::basic::from_utf8(&buf)?.to_string()
    }
  };

  let encoding = tokenizer
    .tokenizer
    .encode(text_str, false)
    .map_err(|e| anyhow!("{:?}", e))?;

  Ok(json!({
    "ids": encoding.get_ids(),
    "offsetMapping": encoding.get_offsets()
  }))
}

/// Credit: Hugging Face
/// Copied this to use non-blocking reqwest client
pub async fn from_pretrained<S: AsRef<str>>(
  identifier: S,
  params: Option<FromPretrainedParameters>,
) -> Result<PathBuf> {
  let identifier: &str = identifier.as_ref();

  let valid_chars = ['-', '_', '.', '/'];
  let is_valid_char = |x: char| x.is_alphanumeric() || valid_chars.contains(&x);

  let valid = identifier.chars().all(is_valid_char);
  let valid_chars_stringified = valid_chars
    .iter()
    .fold(vec![], |mut buf, x| {
      buf.push(format!("'{}'", x));
      buf
    })
    .join(", "); // "'/', '-', '_', '.'"
  if !valid {
    bail!(
          "Model \"{}\" contains invalid characters, expected only alphanumeric or {valid_chars_stringified}",
          identifier
      );
  }
  let params = params.unwrap_or_default();
  let cache_dir = ensure_cache_dir()?;

  let revision = &params.revision;
  let valid_revision = revision.chars().all(is_valid_char);
  if !valid_revision {
    bail!(
          "Revision \"{}\" contains invalid characters, expected only alphanumeric or {valid_chars_stringified}",
          revision
      );
  }

  let filepath = cache_dir.join(identifier).join(revision);
  if filepath.exists() {
    return Ok(filepath);
  }
  std::fs::create_dir_all(&filepath.parent().unwrap())?;

  // Build a custom HTTP Client using our user-agent and custom headers
  let mut headers = HeaderMap::new();
  if let Some(ref token) = params.auth_token {
    headers.insert(
      "Authorization",
      HeaderValue::from_str(&format!("Bearer {}", token))?,
    );
  }
  let download_client = reqwest::Client::builder()
    .user_agent("arena/tokenizer")
    .default_headers(headers);

  let url_to_download = format!(
    "https://huggingface.co/{}/resolve/{}/tokenizer.json",
    identifier, revision,
  );

  let response = download_client.build()?.get(url_to_download).send().await?;
  let mut file = File::create(filepath.clone())?;
  file.write_all(&response.bytes().await?)?;

  Ok(filepath)
}

fn cache_dir() -> PathBuf {
  if let Ok(path) = std::env::var("TOKENIZERS_CACHE") {
    PathBuf::from(path)
  } else {
    let mut dir = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
    dir.push("huggingface");
    dir.push("tokenizers");
    dir
  }
}

/// Returns a directory to be used as cache, creating it if it doesn't exist
///
/// Cf `cache_dir()` to understand how the cache dir is selected.
fn ensure_cache_dir() -> std::io::Result<PathBuf> {
  let dir = cache_dir();
  std::fs::create_dir_all(&dir)?;
  Ok(dir)
}
