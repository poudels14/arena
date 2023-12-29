mod core;
mod loaders;
mod resolver;
mod transpiler;

pub mod config;
pub mod env;
pub mod extensions;
pub mod permissions;
pub mod utils;

pub use crate::core::{IsolatedRuntime, RuntimeOptions};

pub mod buildtools {
  pub use crate::loaders::FileModuleLoader;
  pub use crate::resolver::FilePathResolver;
  pub mod transpiler {
    pub use crate::transpiler::{
      jsx_analyzer::JsxAnalyzer, BabelTranspiler, SwcTranspiler,
    };
  }
}

pub mod deno {
  pub use deno_core as core;
}
