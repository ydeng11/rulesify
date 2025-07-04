use crate::models::rule::UniversalRule;
use crate::store::RuleStore;
use anyhow::Result;
use std::collections::HashMap;

pub struct MemoryStore {
    rules: HashMap<String, UniversalRule>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RuleStore for MemoryStore {
    fn load_rule(&self, id: &str) -> Result<Option<UniversalRule>> {
        Ok(self.rules.get(id).cloned())
    }

    fn save_rule(&self, rule: &UniversalRule) -> Result<()> {
        // Note: This would need interior mutability in practice
        // For now, this is just a skeleton implementation
        println!("Would save rule: {}", rule.id);
        Ok(())
    }

    fn list_rules(&self) -> Result<Vec<String>> {
        let mut rule_ids: Vec<String> = self.rules.keys().cloned().collect();
        rule_ids.sort();
        Ok(rule_ids)
    }

    fn delete_rule(&self, id: &str) -> Result<()> {
        // Note: This would need interior mutability in practice
        println!("Would delete rule: {}", id);
        Ok(())
    }
} 