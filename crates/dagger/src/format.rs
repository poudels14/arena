use anyhow::{bail, Result};
use clap::Parser;
use colored::*;
use common;
use common::utils::fs::has_file_in_file_tree;
use std::env;
use std::path::{Path, PathBuf};
use std::process;
use tracing::info;

#[derive(Parser, Debug)]
pub struct Command {
  /// Run formatter with verbose mode
  #[arg(short, long)]
  verbose: bool,
}

#[derive(Debug)]
enum ProjectType {
  Rust,
  NodeJs,
}

impl Command {
  pub async fn execute(&self) -> Result<()> {
    let current_dir = env::current_dir().unwrap();
    let mut project_type: Option<ProjectType> = None;
    let mut root_dir: Option<PathBuf> = None;

    if let Some(p) = has_file_in_file_tree(Some(&current_dir), "package.json") {
      project_type = Some(ProjectType::NodeJs);
      root_dir = Some(p);
    } else if let Some(p) =
      has_file_in_file_tree(Some(&current_dir), "Cargo.toml")
    {
      project_type = Some(ProjectType::Rust);
      root_dir = Some(p);
    }

    match project_type {
      Some(project_type) => {
        info!("Detected project type: {:?}", project_type);
        let child = match project_type {
          ProjectType::NodeJs => {
            info!(
              "Formatting directory [formatter=prettier]: {}",
              current_dir.to_string_lossy()
            );
            Some(self.format_using_prettier(&current_dir, &root_dir.unwrap()))
          }
          ProjectType::Rust => {
            info!(
              "Formatting directory [formatter=cargo fmt]: {}",
              current_dir.to_string_lossy()
            );
            Some(self.cargo_format(&current_dir, &root_dir.unwrap()))
          }
        };

        child.unwrap().wait()
      }
      None => {
        bail!("Couldn't detect project type!");
      }
    }?;

    println!("{}", "Done!".green().bold());
    Ok(())
  }

  /// Formats the js project using pnpm prettier
  /// Searches for the .prettierignore in the current directory as well as up the
  /// file tree of the current directory
  fn format_using_prettier(
    &self,
    current_dir: &Path,
    project_dir: &Path,
  ) -> process::Child {
    let mut args = vec!["prettier".to_owned(), "-w".to_owned()];

    if let Some(prettier_config_dir) =
      has_file_in_file_tree(Some(&current_dir), ".prettierrc")
    {
      let config_file = format!(
        "{}/{}",
        prettier_config_dir.to_string_lossy(),
        ".prettierrc"
      );
      info!("Using prettier config: {}", config_file);
      args.push("--config".to_owned());
      args.push(config_file);
    }

    if let Some(prettier_ignore_dir) =
      has_file_in_file_tree(Some(&current_dir), ".prettierignore")
    {
      let ignore_file = format!(
        "{}/{}",
        prettier_ignore_dir.to_string_lossy(),
        ".prettierignore"
      );
      info!("Using prettier ignore: {}", ignore_file);
      args.push("--ignore-path".to_owned());
      args.push(ignore_file);
    }

    process::Command::new("pnpm")
      .current_dir(project_dir)
      .args(args)
      .arg(format!("{}", current_dir.to_string_lossy()))
      .spawn()
      .expect("failed to execute pnpm")
  }

  /// Formats the rust project using `cargo fmt`
  fn cargo_format(
    &self,
    _current_dir: &Path,
    project_dir: &Path,
  ) -> process::Child {
    let mut args = vec!["fmt".to_owned()];

    if self.verbose {
      args.push("-v".to_owned());
    }

    process::Command::new("cargo")
      .current_dir(project_dir)
      .args(args)
      .spawn()
      .expect("failed to execute pnpm")
  }
}
