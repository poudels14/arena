use crate::ArenaConfig;
use anyhow::{anyhow, bail, Result};
use common::fs::has_file_in_file_tree;
use common::node::{Package, TsConfig};
use log::debug;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;

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

  config.create_dir(&workspace_dir.join("./src/queries"))?;
  config.create_dir(&workspace_dir.join("./src/apps"))?;

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
    debug!("Adding arena.config.yaml");
    let mut arena_config: ArenaConfig =
      serde_yaml::from_str(include_str!("../template/arena.config.yaml"))?;
    arena_config.name = self.name.clone();
    self.create_file(
      "arena.config.yaml",
      serde_yaml::to_string(&arena_config)?.as_bytes(),
    )?;

    debug!("Adding package.json");
    let mut package: Package =
      serde_json::from_str(include_str!("../template/package.json"))?;
    package.name = self.name.clone();
    self.create_file(
      "package.json",
      serde_json::to_string_pretty(&package)
        .map_err(|e| anyhow!("{:?}", e))?
        .as_bytes(),
    )?;

    debug!("Adding tsconfig.json");
    let ts_config: TsConfig =
      serde_json::from_str(include_str!("../template/tsconfig.json"))?;
    self.create_file(
      "tsconfig.json",
      serde_json::to_string_pretty(&ts_config)
        .map_err(|e| anyhow!("{:?}", e))?
        .as_bytes(),
    )?;

    self.create_file(
      "entry-server.tsx",
      include_bytes!("../template/entry-server.tsx"),
    )?;
    self.create_file(".gitignore", include_bytes!("../template/.gitignore"))?;

    Ok(())
  }
}
