pub mod dependency;
pub mod error;
pub mod reconcile;

pub use dependency::check_all_dependencies;
pub use error::{Result, RulesifyError};
pub use reconcile::{reconcile_global_config, reconcile_project_config, skill_exists_on_disk};

#[cfg(test)]
mod reconcile_tests;
