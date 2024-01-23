use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use anyhow::{bail, Result};
use deno_core::{JsBuffer, Resource, ResourceId, ToJsBuffer};
use runtime::deno::core::{op2, OpState};
use s3::creds::Credentials;
use s3::{Bucket, BucketConfiguration, Region};
use serde::{Deserialize, Serialize};

use serde_json::json;
use types::Object;

mod types;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct S3Client {
  region: Region,
  credentials: Credentials,
  with_path_style: bool,
}

impl Resource for S3Client {}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateBucketRequest {
  name: String,
  public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListBucketOptions {
  prefix: Option<String>,
  delimiter: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ListBucketReponse {
  pub objects: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PubObjectRequest {
  content: JsBuffer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct GetObjectResponse {
  headers: HashMap<String, String>,
  content: ToJsBuffer,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HeadObjectResponse {
  pub accept_ranges: Option<String>,
  pub content_disposition: Option<String>,
  pub content_encoding: Option<String>,
  pub content_length: Option<i64>,
  pub content_type: Option<String>,
  pub e_tag: Option<String>,
  pub expiration: Option<String>,
  pub expires: Option<String>,
  pub last_modified: Option<String>,
  pub metadata: HashMap<String, String>,
  pub parts_count: Option<i64>,
  pub restore: Option<String>,
  pub storage_class: Option<String>,
  pub version_id: Option<String>,
}

#[op2]
#[serde]
pub(crate) fn op_cloud_s3_create_client(
  state: &mut OpState,
  #[serde] client: S3Client,
) -> Result<ResourceId> {
  let id = state.resource_table.borrow_mut().add(client);
  Ok(id)
}

#[op2(async)]
#[serde]
pub(crate) async fn op_cloud_s3_create_bucket(
  state: Rc<RefCell<OpState>>,
  #[smi] id: ResourceId,
  #[serde] request: CreateBucketRequest,
) -> Result<serde_json::Value> {
  let client = state.borrow().resource_table.get::<S3Client>(id)?;
  let config = match request.public.unwrap_or(false) {
    true => BucketConfiguration::public(),
    false => BucketConfiguration::private(),
  };

  let bucket = if client.with_path_style {
    Bucket::create_with_path_style(
      &request.name,
      client.region.clone(),
      client.credentials.clone(),
      config,
    )
    .await
  } else {
    Bucket::create(
      &request.name,
      client.region.clone(),
      client.credentials.clone(),
      config,
    )
    .await
  }?;

  if bucket.response_code != 200 {
    bail!("Error creating bucket: {}", bucket.response_text);
  }
  Ok(json!({
    "name": bucket.bucket.name
  }))
}

#[op2(async)]
#[serde]
pub(crate) async fn op_cloud_s3_list_bucket(
  state: Rc<RefCell<OpState>>,
  #[smi] id: ResourceId,
  #[string] name: String,
  #[serde] request: ListBucketOptions,
) -> Result<ListBucketReponse> {
  let client = state.borrow().resource_table.get::<S3Client>(id)?;
  let region = client.region.clone();
  let credentials = client.credentials.clone();
  let bucket = Bucket::new(&name, region, credentials)?;
  let bucket = match client.with_path_style {
    true => bucket.with_path_style(),
    false => bucket,
  };

  let list = bucket
    .list(request.prefix.unwrap_or("/".to_owned()), request.delimiter)
    .await?;

  let objects = list
    .into_iter()
    .flat_map(|bucket| bucket.contents)
    .map(|object| Object {
      last_modified: object.last_modified,
      e_tag: object.e_tag,
      storage_class: object.storage_class,
      key: object.key,
      size: object.size,
    })
    .collect();

  Ok(ListBucketReponse { objects })
}

#[op2(async)]
pub async fn op_cloud_s3_put_object(
  state: Rc<RefCell<OpState>>,
  #[smi] id: ResourceId,
  #[string] bucket_name: String,
  #[string] path: String,
  #[serde] request: PubObjectRequest,
) -> Result<()> {
  let client = state.borrow().resource_table.get::<S3Client>(id)?.clone();
  let region = client.region.clone();
  let credentials = client.credentials.clone();

  let bucket = Bucket::new(&bucket_name, region, credentials)?;
  let bucket = match client.with_path_style {
    true => bucket.with_path_style(),
    false => bucket,
  };

  let response = bucket.put_object(path, request.content.deref()).await?;
  if response.status_code() != 200 {
    bail!("Error: {}", response.as_str().unwrap())
  }
  Ok(())
}

#[op2(async)]
#[serde]
pub async fn op_cloud_s3_head_object(
  state: Rc<RefCell<OpState>>,
  #[smi] id: ResourceId,
  #[string] bucket_name: String,
  #[string] path: String,
) -> Result<HeadObjectResponse> {
  let client = state.borrow().resource_table.get::<S3Client>(id)?.clone();
  let region = client.region.clone();
  let credentials = client.credentials.clone();

  let bucket = Bucket::new(&bucket_name, region, credentials)?;
  let bucket = match client.with_path_style {
    true => bucket.with_path_style(),
    false => bucket,
  };

  let (response, status) = bucket.head_object(path).await?;
  if status != 200 {
    bail!("Error: status code = {}", status);
  }
  Ok(HeadObjectResponse {
    accept_ranges: response.accept_ranges,
    content_disposition: response.content_disposition,
    content_encoding: response.content_encoding,
    content_length: response.content_length,
    content_type: response.content_type,
    e_tag: response.e_tag,
    expiration: response.expiration,
    expires: response.expires,
    last_modified: response.last_modified,
    metadata: response.metadata.unwrap_or_default(),
    parts_count: response.parts_count,
    restore: response.restore,
    storage_class: response.storage_class,
    version_id: response.version_id,
  })
}

#[op2(async)]
#[serde]
pub async fn op_cloud_s3_get_object(
  state: Rc<RefCell<OpState>>,
  #[smi] id: ResourceId,
  #[string] bucket_name: String,
  #[string] path: String,
) -> Result<GetObjectResponse> {
  let client = state.borrow().resource_table.get::<S3Client>(id)?.clone();
  let region = client.region.clone();
  let credentials = client.credentials.clone();

  let bucket = Bucket::new(&bucket_name, region, credentials)?;
  let bucket = match client.with_path_style {
    true => bucket.with_path_style(),
    false => bucket,
  };

  let response = bucket.get_object(path).await?;
  if response.status_code() != 200 {
    bail!("Error: {}", response.as_str().unwrap())
  }
  Ok(GetObjectResponse {
    content: ToJsBuffer::from(response.bytes().to_vec()),
    headers: response.headers(),
  })
}
