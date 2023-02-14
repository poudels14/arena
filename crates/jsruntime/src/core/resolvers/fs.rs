use crate::core::loaders::FsModuleLoader;
use anyhow::{anyhow, bail, Result};
use common::node::Package;
use deno_core::ModuleResolutionError::{
  InvalidBaseUrl, InvalidPath, InvalidUrl,
};
use deno_core::{ModuleResolutionError, ModuleSpecifier};
use log::debug;
use std::path::PathBuf;
use url::{ParseError, Url};

const SUPPORTED_EXTENSIONS: [&'static str; 5] =
  ["ts", "tsx", "js", "jsx", "json"];

pub(crate) fn resolve_import(
  loader: &FsModuleLoader,
  specifier: &str,
  base: &str,
) -> Result<ModuleSpecifier, ModuleResolutionError> {
  debug!(
    "Resolving import: specifier = {:?}, base = {:?}",
    specifier, base
  );

  let url = match Url::parse(specifier) {
    // 1. Apply the URL parser to specifier.
    //    If the result is not failure, return he result.
    Ok(url) => url,

    // 2. If specifier does not start with the character U+002F SOLIDUS (/),
    //    the two-character sequence U+002E FULL STOP, U+002F SOLIDUS (./),
    //    or the three-character sequence U+002E FULL STOP, U+002E FULL STOP,
    //    U+002F SOLIDUS (../), resolve from npm packages
    Err(ParseError::RelativeUrlWithoutBase)
      if !(specifier.starts_with('/')
        || specifier.starts_with("./")
        || specifier.starts_with("../")) =>
    {
      let maybe_referrer = if base.is_empty() {
        None
      } else {
        Some(base.to_string())
      };
      return super::npm::resolve_module(loader, specifier, maybe_referrer);
    }

    // 3. Return the result of applying the URL parser to specifier with base
    //    URL as the base URL.
    Err(ParseError::RelativeUrlWithoutBase) => {
      let filepath = Url::parse(base)
        .map_err(InvalidBaseUrl)?
        .join(specifier)
        .map_err(InvalidBaseUrl)?
        .to_file_path()
        .map_err(|_| InvalidBaseUrl(ParseError::RelativeUrlWithoutBase))?;

      let maybe_package = super::npm::load_package_json_in_dir(&filepath).ok();
      load_as_file(&filepath)
        .or_else(|e| {
          debug!("error loading as file: {:?}", e);
          load_as_directory(&filepath, &maybe_package)
        })
        .map_err(|_| InvalidPath(filepath))?
    }

    // If parsing the specifier as a URL failed for a different reason than
    // it being relative, always return the original error. We don't want to
    // return `ImportPrefixMissing` or `InvalidBaseUrl` if the real
    // problem lies somewhere else.
    Err(err) => return Err(InvalidUrl(err)),
  };

  Ok(url)
}

pub fn load_as_file(file: &PathBuf) -> Result<ModuleSpecifier> {
  debug!(
    "load_as_file path: {:?}, supported extensions: {:?}",
    file, SUPPORTED_EXTENSIONS
  );

  for ext in SUPPORTED_EXTENSIONS {
    let file_with_extension = file.with_extension(ext);
    if file_with_extension.exists() {
      debug!("file with extension exists: {:?}", file_with_extension);
      return Url::from_file_path(file_with_extension)
        .map_err(|e| anyhow!("{:?}", e));
    }
  }
  bail!("file not found: {:?}", file);
}

pub fn load_index(path: &PathBuf) -> Result<ModuleSpecifier> {
  debug!("checking index file at: {:?}", path);
  load_as_file(&path.join("index"))
}

/// if the directory contains package.json, package arg is not None
pub fn load_as_directory(
  path: &PathBuf,
  maybe_package: &Option<Package>,
) -> Result<ModuleSpecifier> {
  debug!("load_as_directory path: {:?}", path);

  if let Some(package) = maybe_package.as_ref() {
    if let Some(main) = &package.main {
      let main_file = path.join(main);
      return load_as_file(&main_file).or_else(|_| load_index(&main_file));
    }
  };
  debug!("package.json not found in {:?}", &path);
  load_index(&path)
}
