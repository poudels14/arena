use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{anyhow, bail, Result};
use deno_core::ModuleResolutionError::InvalidBaseUrl;
use deno_core::ModuleResolutionError::{
  ImportPrefixMissing, InvalidPath, InvalidUrl,
};
use deno_core::{normalize_path, ModuleResolutionError, ModuleSpecifier};
use indexmap::IndexMap;
use indexmap::IndexSet;
use serde_json::Value;
use tracing::{debug, error, instrument, Level};
use url::{ParseError, Url};

use super::{ResolutionType, Resolver};
use crate::config::node::{Package, ResolverConfig};

const SUPPORTED_EXTENSIONS: [&'static str; 9] = [
  "ts", "tsx", "js", "mjs", "jsx", "json", "cjs", "css", ".scss",
];
static DEFAULT_EXPORT_CONDITIONS: [&'static str; 1] = ["node"];

#[derive(Debug)]
pub struct ParsedSpecifier {
  package_name: String,
  sub_path: String,
}

#[derive(Default)]
pub(crate) struct ResolverCache {
  pub node_module_dirs: IndexMap<String, Vec<PathBuf>>,
  /// Map of npm package name -> (package.json, path to package.json)
  pub packages: IndexMap<String, (Package, PathBuf)>,
  /// Map of resolved module path -> npm package name
  pub resolved_path_to_package_name: IndexMap<String, String>,
}

pub struct FilePathResolver {
  /// The root directory of the project. It's usually where package.json is
  project_root: PathBuf,
  config: ResolverConfig,
  pub(crate) cache: Rc<RefCell<ResolverCache>>,
}

impl FilePathResolver {
  pub fn new(project_root: PathBuf, config: ResolverConfig) -> Self {
    Self {
      project_root,
      config,
      cache: Rc::new(RefCell::new(ResolverCache {
        ..Default::default()
      })),
    }
  }
}

impl Resolver for FilePathResolver {
  #[instrument(skip(self), level = "trace")]
  fn resolve(
    &self,
    specifier: &str,
    base: &str,
    resolution: ResolutionType,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    let specifier = self.resolve_alias(specifier);
    let url = match Url::parse(&specifier) {
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
        return self.resolve_npm_module(&specifier, maybe_referrer, resolution);
      }

      // 3. Return the result of applying the URL parser to specifier with base
      //    URL as the base URL.
      Err(ParseError::RelativeUrlWithoutBase) => {
        let filepath = Url::parse(base)
          .map_err(InvalidBaseUrl)?
          .join(&specifier)
          .map_err(InvalidBaseUrl)?
          .to_file_path()
          .map_err(|_| InvalidBaseUrl(ParseError::RelativeUrlWithoutBase))?;

        load_as_file(&filepath)
          .or_else(|e| {
            debug!("error loading as file: {:?}", e);
            let maybe_package = load_package_json_in_dir(&filepath).ok();
            load_as_directory(&filepath, &maybe_package)
          })
          .and_then(|p| self.convert_to_url(p))
          .map_err(|_| InvalidPath(filepath))?
      }

      // If parsing the specifier as a URL failed for a different reason than
      // it being relative, always return the original error. We don't want to
      // return `ImportPrefixMissing` or `InvalidBaseUrl` if the real
      // problem lies somewhere else.
      Err(err) => {
        error!("Parsing specifier failed! specifier = {specifier:?}");
        return Err(InvalidUrl(err));
      }
    };

    Ok(url)
  }
}

impl FilePathResolver {
  fn resolve_alias(&self, specifier: &str) -> String {
    let alias = &self.config.alias;
    for k in alias.keys() {
      let alias_len = k.len();
      if specifier.starts_with(k)
        && (specifier.len() == alias_len
          || (specifier.len() > alias_len
            && &specifier[alias_len..alias_len + 1] == "/"))
      {
        let value = alias.get(k).unwrap();
        debug!("matched alias: {}={}", k, value);
        return format!(
          "{}{}",
          if value.starts_with(".") {
            format!("{}", self.project_root.join(value).to_str().unwrap())
          } else {
            value.to_string()
          },
          &specifier[k.len()..]
        );
      }
    }
    specifier.to_owned()
  }

  pub(super) fn convert_to_url(&self, path: PathBuf) -> Result<Url> {
    let path = match self.config.preserve_symlink.unwrap_or(false) {
      true => path,
      false => {
        // Note(sagar): canonicalize when preserve symlink is false so that
        // pnpm works
        path.canonicalize()?
      }
    };

    Url::from_file_path(&path)
      .map_err(|()| anyhow!("failed to convert {:?} to file url", path))
  }

  #[tracing::instrument(skip_all, level = "trace")]
  pub(crate) fn resolve_npm_module(
    &self,
    specifier: &str,
    maybe_referrer: Option<String>,
    resolution: ResolutionType,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    match maybe_referrer.as_ref() {
      Some(referrer) => {
        let parsed_specifier = parse_specifier(&specifier);
        let mut cache = self.cache.borrow_mut();
        let root = self.project_root.clone();

        // Note(sagar): if a module is deduped, it needs to be resolved from
        // ${project root}/node_modules. using `./` specifier is a hack to
        // force resolver to use same node_modules directory for all deduped
        // modules
        let referrer_url;
        let referrer_to_use =
          if self.config.dedupe.contains(&parsed_specifier.package_name) {
            referrer_url = Url::from_file_path(&root)
              .map_err(|_| InvalidPath(root.clone()))?;
            referrer_url.as_str()
          } else {
            referrer
          };
        let directories = match cache.node_module_dirs.get(referrer_to_use) {
          Some(dir) => dir,
          None => {
            let directories = Self::valid_node_modules_paths(referrer_to_use)?;

            #[cfg(debug_assertions)]
            {
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
            }
            cache
              .node_module_dirs
              .insert(referrer_to_use.to_string(), directories);
            cache.node_module_dirs.get(referrer_to_use).unwrap()
          }
        };

        for node_modules_dir in directories {
          debug!("using node_module in: {}", &node_modules_dir.display());
          let maybe_package = load_package_json_in_dir(
            &node_modules_dir.join(&parsed_specifier.package_name),
          )
          .ok();

          if resolution == ResolutionType::Require {
            if let Some(main) =
              maybe_package.as_ref().and_then(|p| p.main.clone())
            {
              let main_path = node_modules_dir.join(specifier).join(&main);
              return self
                .convert_to_url(main_path)
                .map_err(|_| InvalidPath(node_modules_dir.join(main)));
            }
          }

          let resolved = self
            .load_npm_package(
              &node_modules_dir,
              &parsed_specifier,
              &maybe_package,
            )
            .or_else(|e| {
              debug!("error loading npm package export: {}", e);
              load_as_file(&node_modules_dir.join(specifier))
            })
            .or_else(|e| {
              debug!("error loading as file: {}", e);
              load_as_directory(
                &node_modules_dir.join(specifier),
                &maybe_package,
              )
            })
            .or_else(|e| {
              debug!("error loading as directory: {}", e);
              self.load_from_imports(
                &specifier,
                maybe_package
                  .as_ref()
                  .map(|p| (p, node_modules_dir))
                  .or_else(|| {
                    cache.resolved_path_to_package_name.get(referrer).and_then(
                      |package_name| {
                        cache
                          .packages
                          .get(package_name)
                          .as_ref()
                          .map(|(package, dir)| (package, dir))
                      },
                    )
                  }),
              )
            })
            .and_then(|p| self.convert_to_url(p));

          if let Ok(resolved) = resolved {
            if let Some(package) = maybe_package {
              if !cache.packages.contains_key(&package.name) {
                let used_node_modules_dir = node_modules_dir.clone();
                cache.packages.insert(
                  package.name.clone(),
                  (package.clone(), used_node_modules_dir),
                );
              }
              cache
                .resolved_path_to_package_name
                .insert(resolved.as_str().to_owned(), package.name.clone());
            } else {
              let referrer_package =
                cache.resolved_path_to_package_name.get(referrer);

              if let Some(referrer_package) = referrer_package {
                let referrer_package = referrer_package.to_string();
                cache
                  .resolved_path_to_package_name
                  .insert(resolved.as_str().to_owned(), referrer_package);
              }
            }
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
  ) -> Result<PathBuf> {
    let package: &Package =
      maybe_package.as_ref().ok_or(anyhow!("not a npm package"))?;

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

  /// Some packages have "imports" field in package.json that maps
  /// specifier to the filename and the aliased specifier is used
  /// to import modules; this is used to load those "aliased" modules
  fn load_from_imports(
    &self,
    specifier: &str,
    package: Option<(&Package, &PathBuf)>,
  ) -> Result<PathBuf> {
    if let Some((package, base_dir)) = package {
      let resolved_path = package
        .imports
        .as_ref()
        .and_then(|imports| imports.get(specifier))
        .and_then(|conditional_imports| {
          get_matching_export(conditional_imports, &self.config.conditions).ok()
        })
        .and_then(|alias| {
          Some(normalize_path(base_dir.join(&package.name).join(alias)))
        });

      if let Some(resolved_path) = resolved_path {
        if resolved_path.exists() {
          return Ok(resolved_path);
        }
      }
    }
    bail!("package.json not available to load \"imports\" from");
  }

  fn load_package_exports(
    &self,
    base_dir: &PathBuf,
    specifier: &ParsedSpecifier,
    package: &Package,
  ) -> Result<PathBuf> {
    // TODO(sagar): handle other exports type
    let resolved_path = normalize_path(
      base_dir
        .join(&package.name)
        .join(self.get_package_json_export(&package, &specifier.sub_path)?),
    );

    debug!("resolved path: {:?}", resolved_path);

    if resolved_path.exists() {
      return Ok(resolved_path);
    }
    bail!("package export not found for specifier: {:?}", &specifier);
  }

  // reference: https://nodejs.org/api/modules.html#all-together
  // TODO(sagar): is it possible to check for path permission when
  // resolving instead of when loading the resolved files?
  fn valid_node_modules_paths(
    referrer: &str,
  ) -> Result<Vec<PathBuf>, ModuleResolutionError> {
    if !referrer.starts_with("file://") {
      error!("invalid module referrer: expected it to start with 'file://' but is: {:?}", referrer);
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
      if dir.is_dir() {
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

  #[tracing::instrument(skip(self, package), level = "trace")]
  fn get_package_json_export(
    &self,
    package: &Package,
    specifier_subpath: &str,
  ) -> Result<String> {
    match package.exports.as_ref() {
      Some(exports) => {
        if let Some(export) = exports.as_str() {
          return Ok(export.to_string());
        }
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
    true => format!(".{sub_path}"),
    false => format!("./{sub_path}"),
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
      || condition.eq("import")
      // Note(sp): if explicit conditions isn't passed, use default conditions
      // that uses node modules
      || (conditions.is_empty()
        && DEFAULT_EXPORT_CONDITIONS.contains(&condition.as_str()))
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

#[tracing::instrument(level = "trace")]
pub fn load_as_file(file: &PathBuf) -> Result<PathBuf> {
  if file.is_file() {
    return Ok(file.clone());
  }

  for ext in SUPPORTED_EXTENSIONS {
    let ext = match file.extension().and_then(|e| e.to_str()) {
      // Note(sagar): if file already has extension that's not
      // in SUPPORTED_EXTENSIONS, combine the extensions
      // the is needed to load files with multiple `.` in the filename
      Some(e) if !SUPPORTED_EXTENSIONS.contains(&e) => format!("{}.{}", e, ext),
      _ => ext.to_owned(),
    };
    let file_with_extension = file.with_extension(&ext);
    if file_with_extension.exists() {
      debug!("matched extension: {}", ext);
      return Ok(file_with_extension);
    }
  }
  bail!("file not found: {:?}", file);
}

pub fn load_index(path: &PathBuf) -> Result<PathBuf> {
  debug!("checking index file at: {:?}", path);
  load_as_file(&path.join("index"))
}

/// if the directory contains package.json, package arg is not None
pub fn load_as_directory(
  path: &PathBuf,
  maybe_package: &Option<Package>,
) -> Result<PathBuf> {
  debug!("load_as_directory path: {:?}", path);

  if let Some(package) = maybe_package.as_ref() {
    // Note(sagar): prioritize ESM module
    if let Some(module) = &package.module {
      let module_file = path.join(module);
      return load_as_file(&module_file).or_else(|_| load_index(&module_file));
    }

    if let Some(main) = &package.main {
      let main_file = path.join(main);
      return load_as_file(&main_file).or_else(|_| load_index(&main_file));
    }
  };
  debug!("package.json not found in {:?}", &path);
  load_index(&path)
}
