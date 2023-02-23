use super::fs::FsModuleResolver;
use super::{fs, ParsedSpecifier};
use anyhow::{anyhow, bail, Result};
use common::node::Package;
use deno_core::ModuleResolutionError::{
  ImportPrefixMissing, InvalidPath, InvalidUrl,
};
use deno_core::{normalize_path, ModuleResolutionError, ModuleSpecifier};
use indexmap::{indexset, IndexSet};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::{debug, Level};
use url::{ParseError, Url};

static ORDERED_EXPORT_CONDITIONS: Lazy<IndexSet<&str>> =
  Lazy::new(|| indexset!["import"]);

impl FsModuleResolver {
  #[tracing::instrument(skip_all)]
  pub(crate) fn resolve_npm_module(
    &self,
    specifier: &str,
    maybe_referrer: Option<String>,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    match maybe_referrer.as_ref() {
      Some(referrer) => {
        let mut cache = self.cache.borrow_mut();

        let directories = match cache.node_module_dirs.get(referrer) {
          Some(dir) => dir,
          None => {
            let directories = Self::valid_node_modules_paths(referrer)?;
            let root = self.project_root.clone();
            let relative_dirs = directories
              .iter()
              .map(|d| {
                pathdiff::diff_paths::<&PathBuf, &PathBuf>(d, &root).unwrap()
              })
              .collect::<Vec<PathBuf>>();
            debug!(
              "caching resolved node_modules directories: {:?}",
              relative_dirs
            );
            cache
              .node_module_dirs
              .insert(referrer.to_string(), directories);
            cache.node_module_dirs.get(referrer).unwrap()
          }
        };

        let parsed_specifier = parse_specifier(&specifier);
        for dir_path in directories {
          debug!("using node_module in: {}", &dir_path.display());
          let maybe_package = load_package_json_in_dir(
            &dir_path.join(&parsed_specifier.package_name),
          )
          .ok();
          if let Ok(resolved) = self
            .load_npm_package(&dir_path, &parsed_specifier, &maybe_package)
            .or_else(|e| {
              debug!("error loading npm package export: {:?}", e);
              fs::load_as_file(&dir_path.join(specifier))
            })
            .or_else(|e| {
              debug!("error loading as file: {:?}", e);
              fs::load_as_directory(&dir_path.join(specifier), &maybe_package)
            })
          {
            return Ok(resolved);
          }
        }
        Err(InvalidPath(Path::new(referrer).join(specifier)))
      }
      None => Err(ImportPrefixMissing(specifier.to_string(), maybe_referrer)),
    }
  }

  fn load_npm_package(
    &self,
    base_dir: &PathBuf,
    specifier: &ParsedSpecifier,
    maybe_package: &Option<Package>,
  ) -> Result<ModuleSpecifier> {
    let package: &Package =
      maybe_package.as_ref().ok_or(anyhow!("not a npm package"))?;

    debug!("package.json loaded");

    let package_export =
      self.load_package_exports(base_dir, specifier, &package);

    if package_export.is_ok() {
      return package_export;
    }

    // TODO(sagar): if package_json.module is present, use that

    bail!(
      "module not found for specifier: {}{}",
      &specifier.package_name,
      &specifier.sub_path[1..]
    );
  }

  fn load_package_exports(
    &self,
    base_dir: &PathBuf,
    specifier: &ParsedSpecifier,
    package: &Package,
  ) -> Result<ModuleSpecifier> {
    // TODO(sagar): handle other exports type
    let resolved_path = normalize_path(
      base_dir
        .join(&package.name)
        .join(self.get_package_json_export(&package, &specifier.sub_path)?),
    );

    debug!("resolved path: {:?}", resolved_path);

    if resolved_path.exists() {
      return Url::from_file_path(&resolved_path)
        .map_err(|e| anyhow!("{:?}", e));
    }
    bail!("package export not found for specifier: {:?}", &specifier);
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
    &self,
    package: &Package,
    specifier_subpath: &str,
  ) -> Result<String> {
    match package.exports.as_ref() {
      Some(exports) => {
        // Exports are selected based on this doc:
        // https://webpack.js.org/guides/package-exports/
        if let Some(subpath_export) = exports.get(specifier_subpath) {
          return get_matching_export(subpath_export, &self.config.conditions);
        }

        bail!("not implemented")
      }
      None => bail!("exports field missing in package.json"),
    }
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
    let path = subpath_export.as_str().unwrap().to_string();
    debug!(path, "using export");
    return Ok(path);
  }
  let export = subpath_export.as_object().unwrap();
  for (condition, value) in export.iter() {
    if conditions.contains(condition)
      || ORDERED_EXPORT_CONDITIONS.contains(condition.as_str())
    {
      let span = tracing::span!(Level::DEBUG, "get_matching_export", condition);
      let _enter = span.enter();
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
