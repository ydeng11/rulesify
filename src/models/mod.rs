pub mod config;
pub mod context;
pub mod domain;
pub mod global_config;
pub mod install_action;
pub mod registry;
pub mod repo_metrics;
pub mod skill;
pub mod skill_metadata;

#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod domain_tests;
#[cfg(test)]
mod install_action_tests;
#[cfg(test)]
mod registry_tests;
#[cfg(test)]
mod skill_metadata_tests;
#[cfg(test)]
mod skill_tests;

pub use config::{InstalledSkill, ProjectConfig, Scope};
pub use context::ProjectContext;
pub use domain::Domain;
pub use global_config::{get_global_config_path, GlobalConfig};
pub use install_action::InstallAction;
pub use registry::Registry;
pub use repo_metrics::RepoMetrics;
pub use skill::Skill;
pub use skill_metadata::SkillMetadata;
