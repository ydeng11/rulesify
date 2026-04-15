pub mod cache;
pub mod data;
pub mod fetch;
pub mod source;

#[cfg(test)]
mod data_tests;
#[cfg(test)]
mod source_tests;

pub use cache::RegistryCache;
pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use source::SourceRepo;
