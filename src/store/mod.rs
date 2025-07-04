pub mod file_store;
pub mod memory_store;

use crate::models::rule::UniversalRule;
use anyhow::Result;

pub trait RuleStore {
    fn load_rule(&self, id: &str) -> Result<Option<UniversalRule>>;
    fn save_rule(&self, rule: &UniversalRule) -> Result<()>;
    fn list_rules(&self) -> Result<Vec<String>>;
    fn delete_rule(&self, id: &str) -> Result<()>;
} 