use crate::node::Package;
use anyhow::{anyhow, bail, Result};
use deno_core::ModuleResolutionError::{
  InvalidBaseUrl, InvalidPath, InvalidUrl,
};
use deno_core::{ModuleResolutionError, ModuleSpecifier};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use tracing::{debug, error, instrument};
use url::{ParseError, Url};

const SUPPORTED_EXTENSIONS: [&'static str; 9] = [
  "ts", "tsx", "js", "mjs", "jsx", "json", "cjs", "css", ".scss",
];

#[derive(Default)]
pub(crate) struct ResolverCache {
  pub node_module_dirs: IndexMap<String, Vec<PathBuf>>,
  /// Map of npm package name -> (package.json, path to package.json)
  pub packages: IndexMap<String, (Package, PathBuf)>,
  /// Map of resolved module path -> npm package name
  pub resolved_path_to_package_name: IndexMap<String, String>,
}

pub struct FsModuleResolver {
  /// The root directory of the project. It's usually where package.json is
  pub project_root: PathBuf,

  pub(crate) config: super::Config,

  pub(crate) cache: Rc<RefCell<ResolverCache>>,

  pub(crate) builtin_modules: Vec<String>,
}

impl FsModuleResolver {
  pub fn new(
    project_root: PathBuf,
    config: super::Config,
    builtin_modules: Vec<String>,
  ) -> Self {
    Self {
      project_root,
      config,
      cache: Rc::new(RefCell::new(ResolverCache {
        ..Default::default()
      })),
      builtin_modules,
    }
  }

  #[instrument(skip(self), level = "trace")]
  pub fn resolve(
    &self,
    specifier: &str,
    base: &str,
  ) -> Result<ModuleSpecifier, ModuleResolutionError> {
    // TODO(sagar): cache the resolved module specifier?

    let specifier = specifier.strip_prefix("node:").unwrap_or(specifier);
    let mut specifier = self.resolve_alias(specifier);
    if self.builtin_modules.contains(&specifier)
      || specifier.starts_with("@arena/runtime/")
    {
      debug!("Using builtin module: {specifier}");
      specifier = format!("builtin:///{}", specifier);
    }
    let url = match Url::parse(&specifier) {
      // 1. Apply the URL parser to specifier.
      //    If the result is not failure, return he result.
      Ok(url) => {
        debug!("module resolution not needed");
        url
      }

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
        return self.resolve_npm_module(&specifier, maybe_referrer);
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
            let maybe_package =
              super::npm::load_package_json_in_dir(&filepath).ok();
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
