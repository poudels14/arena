// TODO(sagar): move this to a new crate

pub mod ecma;

pub mod package;
pub use package::Package;

mod tsconfig;
pub use tsconfig::TsConfig;

mod resolver;
pub use resolver::ResolverConfig;
