pub mod cache;
pub mod data;
pub mod fetch;
pub mod github;
pub mod parser;
pub mod source;

#[cfg(test)]
mod data_tests;
#[cfg(test)]
mod github_tests;
#[cfg(test)]
mod parser_tests;
#[cfg(test)]
mod source_tests;

pub use cache::RegistryCache;
pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use github::GitHubClient;
pub use parser::{ParsedSkill, SkillParser};
pub use source::SourceRepo;
