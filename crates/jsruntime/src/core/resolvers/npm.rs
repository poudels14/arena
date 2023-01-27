use super::{fs, ParsedSpecifier};
use anyhow::{anyhow, bail, Result};
use common::node::Package;
use deno_core::ModuleResolutionError::{
  ImportPrefixMissing, InvalidPath, InvalidUrl,
};
use deno_core::{ModuleResolutionError, ModuleSpecifier};
use log::debug;
use std::path::{Path, PathBuf};
use url::{ParseError, Url};

pub fn resolve_module(
  specifier: &str,
  maybe_referrer: Option<String>,
) -> Result<ModuleSpecifier, ModuleResolutionError> {
  match maybe_referrer.as_ref() {
    Some(referrer) => {
      let directories = valid_node_modules_paths(&referrer)?;
      debug!("valid node_modules directories: {:?}", &directories);
      let parsed_specifier = parse_specifier(&specifier);
      for dir_path in directories {
        debug!("checking directory: {:?}", &dir_path);
        if let Ok(resolved) = load_npm_package(&dir_path, &parsed_specifier)
          .or_else(|e| {
            debug!("error loading package exports: {:?}", e);
            fs::load_as_file(&dir_path.join(specifier))
          })
          .or_else(|e| {
            debug!("error loading as file: {:?}", e);
            fs::load_as_directory(&dir_path.join(specifier))
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

  let parsed_specifier = ParsedSpecifier {
    package_name,
    sub_path,
  };

  debug!("specifier parsed: {:?}", &parsed_specifier);

  parsed_specifier
}

fn load_npm_package(
  base_dir: &PathBuf,
  specifier: &ParsedSpecifier,
) -> Result<ModuleSpecifier> {
  let package_path =
    base_dir.join(&specifier.package_name).join("package.json");
  if !package_path.exists() {
    bail!("package.json doesn't exist");
  }
  let content = std::fs::read(package_path).map_err(|e| anyhow!("{}", e))?;
  let package: Package = serde_json::from_str(std::str::from_utf8(&content)?)?;

  debug!("package.json loaded for package: {}", package.name);

  let package_export = load_package_exports(base_dir, specifier, &package);
  if package_export.is_ok() {
    return package_export;
  }

  // TODO(sagar): if package_json.module is present, use that

  bail!("module not found for specifier: {:?}", &specifier);
}

fn load_package_exports(
  base_dir: &PathBuf,
  specifier: &ParsedSpecifier,
  package: &Package,
) -> Result<ModuleSpecifier> {
  // TODO(sagar): handle other exports type
  let module = base_dir
    .join(&package.name)
    .join(get_package_json_export(&package, &specifier.sub_path)?);

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
  package: &Package,
  specifier_subpath: &str,
) -> Result<String> {
  match package.exports.as_ref() {
    Some(exports) => {
      if let Some(export) = exports.get(specifier_subpath) {
        let node_export = export
          .get("node")
          .ok_or(anyhow!("exports.node not found!"))?;
        if node_export.is_string() {
          return Ok(node_export.to_string());
        } else if node_export.is_object() {
          let import = node_export
            .get("import")
            .ok_or(anyhow!("unrecognized export format: {:?}", node_export))?;

          return Ok(
            import
              .get("default")
              .ok_or(anyhow!("default export not found in exports.node"))?
              .as_str()
              .unwrap()
              .to_owned(),
          );
        } else {
          bail!("unrecognized export format: {:?}", node_export);
        }
      }

      bail!("not implemented")
    }
    None => bail!("exports field missing in package.json"),
  }
}
