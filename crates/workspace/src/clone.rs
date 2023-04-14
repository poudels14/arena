use crate::WorkspaceConfig;
use anyhow::{anyhow, bail, Result};
use bytes::Buf;
use common::node::Package;
use common::utils::fs::has_file_in_file_tree;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use tar::Archive;
use tracing::debug;

pub static DEFAULT_WORKSPACE_TEMPLATE: &[u8] =
  include_bytes!(concat!(env!("OUT_DIR"), "/DEFAULT_WORKSPACE_TEMPLATE.tar"));

#[derive(Debug)]
pub struct Config {
  // Name of the new workspace
  pub name: String,

  // Directory to setup workspace in
  pub dir: PathBuf,
}

pub async fn with_default_template(config: &Config) -> Result<()> {
  let workspace_dir = &config.dir;
  let workspace_dir_str = workspace_dir.to_str().unwrap();

  if workspace_dir.exists() {
    bail!("workspace directory already exists: {}", workspace_dir_str);
  } else if let Some(ancestor) =
    has_file_in_file_tree(workspace_dir.parent(), "arena.config.yaml")
  {
    bail!("New workspace can't be created under another workspace, existing workspace at: {:?}", ancestor);
  }

  if config.name.len() < 3 {
    bail!("workspace name should be at least 3 characters long");
  }

  debug!("Creating a new workspace directory: {}", workspace_dir_str);
  config.create_dir(&workspace_dir)?;

  config.add_template_files()?;

  // config.create_dir(&workspace_dir.join("./src/queries"))?;
  // config.create_dir(&workspace_dir.join("./src/apps"))?;

  Ok(())
}

impl Config {
  fn create_dir(&self, path: &PathBuf) -> Result<()> {
    fs::create_dir_all(path).map_err(|e| anyhow!("{:?}", e))
  }

  fn create_file(&self, name: &str, data: &[u8]) -> Result<()> {
    let file = self.dir.join(name);
    let mut file = File::create(file)?;
    file.write_all(data)?;

    Ok(())
  }

  fn add_template_files(&self) -> Result<()> {
    debug!("Adding workspace.config.yaml");
    let mut workspace_config: WorkspaceConfig = toml::from_str(include_str!(
      "../../../js/templates/default/workspace.config.toml"
    ))?;
    workspace_config.name = self.name.clone();
    self.create_file(
      "workspace.config.toml",
      toml::to_string(&workspace_config)?.as_bytes(),
    )?;

    debug!("Adding package.json");
    let mut package: Package = serde_json::from_str(include_str!(
      "../../../js/templates/default/package.json"
    ))?;
    package.name = self.name.clone();
    self.create_file(
      "package.json",
      serde_json::to_string_pretty(&package)
        .map_err(|e| anyhow!("{:?}", e))?
        .as_bytes(),
    )?;

    let mut a = Archive::new(DEFAULT_WORKSPACE_TEMPLATE.reader());
    for file in a.entries()? {
      let mut file = file?;

      let filename = self.dir.join(file.header().path()?);
      fs::create_dir_all(filename.parent().unwrap())?;
      let mut f = File::create(filename)?;

      let content = &mut Vec::with_capacity(file.header().size()?.try_into()?);
      file.read_to_end(content)?;

      f.write_all(content)?;
    }

    Ok(())
  }
}
