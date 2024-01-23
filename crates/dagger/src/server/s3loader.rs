use std::pin::Pin;

use anyhow::{bail, Result};
use awsregion::Region;
use runtime::deno::core::futures::FutureExt;
use runtime::deno::core::{
  FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier,
  ModuleType, ResolutionKind,
};
use s3::creds::Credentials;
use s3::Bucket;
use tracing::instrument;
use url::Url;

pub struct S3ModulerLoader {
  path: String,
  options: S3ModuleLoaderOptions,
}

#[derive(Clone, Debug)]
pub struct S3ModuleLoaderOptions {
  pub bucket: String,
  pub endpoint: String,
  pub credentials: Credentials,
  pub with_path_style: bool,
}

impl S3ModulerLoader {
  pub fn new(path: String, options: S3ModuleLoaderOptions) -> Self {
    Self { path, options }
  }
}

impl ModuleLoader for S3ModulerLoader {
  #[instrument(skip(self), level = "trace")]
  fn resolve(
    &self,
    specifier: &str,
    base: &str,
    resolution: ResolutionKind,
  ) -> Result<ModuleSpecifier> {
    let object_url = Url::parse(&self.options.endpoint)?.join(specifier)?;
    tracing::trace!("resolved: {:?}", object_url.as_ref());

    Ok(object_url)
  }

  fn load(
    &self,
    module_specifier: &ModuleSpecifier,
    _maybe_referrer: Option<&ModuleSpecifier>,
    _is_dyn_import: bool,
  ) -> Pin<Box<ModuleSourceFuture>> {
    let path = self.path.clone();
    let options = self.options.clone();
    let specifier = module_specifier.clone();
    async move {
      let region = Region::Custom {
        region: "unknown".to_owned(),
        endpoint: options.endpoint.clone(),
      };

      let bucket =
        Bucket::new(&options.bucket, region, options.credentials.clone())?;
      let bucket = match options.with_path_style {
        true => bucket.with_path_style(),
        false => bucket,
      };

      let response = bucket.get_object(path.clone()).await?;
      if response.status_code() != 200 {
        bail!("Error: {}", response.as_str().unwrap())
      }
      let code = response.as_str()?;

      Ok(ModuleSource::new(
        ModuleType::JavaScript,
        FastString::Arc(code.into()),
        &specifier,
      ))
    }
    .boxed_local()
  }
}
