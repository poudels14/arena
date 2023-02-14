use super::{fs, ParsedSpecifier};
use crate::core::loaders::FsModuleLoader;
use crate::core::ModuleLoaderConfig;
use anyhow::{anyhow, bail, Result};
use common::node::Package;
use deno_core::ModuleResolutionError::{
  ImportPrefixMissing, InvalidPath, InvalidUrl,
};
use deno_core::{ModuleResolutionError, ModuleSpecifier};
use indexmap::{indexset, IndexSet};
use log::debug;
use once_cell::sync::Lazy;
use serde_json::Value;
use std::path::{Path, PathBuf};
use url::{ParseError, Url};

static ORDERED_EXPORT_CONDITIONS: Lazy<IndexSet<&str>> =
  Lazy::new(|| indexset!["import"]);

pub(crate) fn resolve_module(
  loader: &FsModuleLoader,
  specifier: &str,
  maybe_referrer: Option<String>,
) -> Result<ModuleSpecifier, ModuleResolutionError> {
  match maybe_referrer.as_ref() {
    Some(referrer) => {
      let mut cache = loader.cache.borrow_mut();

      let directories = match cache.node_module_dirs.get(referrer) {
        Some(dir) => dir,
        None => {
          let directories = valid_node_modules_paths(referrer)?;
          debug!(
            "caching resolved node_modules directories: {:?}",
            &directories
          );
          cache
            .node_module_dirs
            .insert(referrer.to_string(), directories);
          cache.node_module_dirs.get(referrer).unwrap()
        }
      };

      let parsed_specifier = parse_specifier(&specifier);
      for dir_path in directories {
        debug!("checking directory: {:?}", &dir_path);
        let maybe_package = load_package_json_in_dir(
          &dir_path.join(&parsed_specifier.package_name),
        )
        .ok();
        if let Ok(resolved) = load_npm_package(
          &loader.config,
          &dir_path,
          &parsed_specifier,
          &maybe_package,
        )
        .or_else(|e| {
          debug!("error loading package exports: {:?}", e);
          fs::load_as_file(&dir_path.join(specifier))
        })
        .or_else(|e| {
          debug!("error loading as file: {:?}", e);
          fs::load_as_directory(&dir_path.join(specifier), &maybe_package)
        }) {
          return Ok(resolved);
        }
      }
      Err(InvalidPath(Path::new(referrer).join(specifier)))
    }
    None => Err(ImportPrefixMissing(specifier.to_string(), maybe_referrer)),
  }
}

fn parse_specifier(specifier: &str) -> ParsedSpecifier {
  let specifier_splits: Vec<&str> = specifier.split("/").collect();
  let (package_name, sub_path) = match specifier.starts_with("@") {
    true => (
      specifier_splits[0..2].join("/"),
      specifier_splits[2..].join("/"),
    ),
    false => (
      specifier_splits[0].to_string(),
      specifier_splits[1..].join("/"),
    ),
  };

  let sub_path = match sub_path.len() == 0 {
    true => ".".to_owned() + &sub_path,
    false => "./".to_owned() + &sub_path,
  };

  ParsedSpecifier {
    package_name,
    sub_path,
  }
}

fn load_npm_package(
  loader_config: &ModuleLoaderConfig,
  base_dir: &PathBuf,
  specifier: &ParsedSpecifier,
  maybe_package: &Option<Package>,
) -> Result<ModuleSpecifier> {
  let package: &Package =
    maybe_package.as_ref().ok_or(anyhow!("not a npm package"))?;

  debug!("package.json loaded for package: {}", package.name);

  let package_export =
    load_package_exports(loader_config, base_dir, specifier, &package);

  if package_export.is_ok() {
    return package_export;
  }

  // TODO(sagar): if package_json.module is present, use that

  bail!("module not found for specifier: {:?}", &specifier);
}

fn load_package_exports(
  loader_config: &ModuleLoaderConfig,
  base_dir: &PathBuf,
  specifier: &ParsedSpecifier,
  package: &Package,
) -> Result<ModuleSpecifier> {
  // TODO(sagar): handle other exports type
  let module = base_dir.join(&package.name).join(get_package_json_export(
    loader_config,
    &package,
    &specifier.sub_path,
  )?);

  debug!("resolved module path: {:?}", module);

  if module.exists() {
    return Url::from_file_path(&module).map_err(|e| anyhow!("{:?}", e));
  }
  bail!("module not found for specifier: {:?}", &specifier);
}

// reference: https://nodejs.org/api/modules.html#all-together
fn valid_node_modules_paths(
  referrer: &str,
) -> Result<Vec<PathBuf>, ModuleResolutionError> {
  if !referrer.starts_with("file://") {
    return Err(InvalidUrl(ParseError::RelativeUrlWithoutBase));
  }

  let mut i = referrer.split("/").count() - 2;
  let mut directories: Vec<PathBuf> = Vec::with_capacity(i);
  let mut referrer = Url::parse(referrer)
    .map_err(|e| InvalidUrl(e))?
    .to_file_path()
    .map_err(|_| InvalidUrl(ParseError::RelativeUrlWithoutBase))?;
  while i > 0 {
    // TODO(sagar): might have to check when i = 0
    // TODO(sagar): idk why we need this
    // if parts[i] == "node_modules" {
    //   break;
    // }

    let dir = referrer.join("node_modules");
    if dir.exists() {
      directories.push(dir);
    }

    i = i - 1usize;
    referrer = referrer
      .parent()
      .map(|p| p.to_path_buf())
      .unwrap_or(referrer);
  }

  Ok(directories)
}

fn get_package_json_export(
  loader_config: &ModuleLoaderConfig,
  package: &Package,
  specifier_subpath: &str,
) -> Result<String> {
  match package.exports.as_ref() {
    Some(exports) => {
      // Exports are selected based on this doc:
      // https://webpack.js.org/guides/package-exports/
      if let Some(subpath_export) = exports.get(specifier_subpath) {
        return get_matching_export(
          subpath_export,
          &loader_config.build_config.export_conditions,
        );
      }

      bail!("not implemented")
    }
    None => bail!("exports field missing in package.json"),
  }
}

pub(crate) fn load_package_json_in_dir(dir: &Path) -> Result<Package> {
  let package_path = dir.join("package.json");
  if !package_path.exists() {
    bail!("package.json doesn't exist");
  }
  let content = std::fs::read(package_path).map_err(|e| anyhow!(e))?;
  serde_json::from_str(std::str::from_utf8(&content)?).map_err(|e| anyhow!(e))
}

fn get_matching_export(
  subpath_export: &Value,
  conditions: &IndexSet<String>,
) -> Result<String> {
  if subpath_export.is_string() {
    return Ok(subpath_export.as_str().unwrap().to_string());
  }
  let export = subpath_export.as_object().unwrap();
  for (key, value) in export.iter() {
    if conditions.contains(key)
      || ORDERED_EXPORT_CONDITIONS.contains(key.as_str())
    {
      if let Ok(result) = get_matching_export(value, conditions) {
        return Ok(result);
      }
    }
  }

  // Note(sagar): always try default export
  return get_matching_export(
    export
      .get("default")
      .ok_or(anyhow!("no matching condition found"))?,
    conditions,
  );
}
