use crate::models::rule::UniversalRule;
use crate::store::RuleStore;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct FileStore {
    rules_directory: PathBuf,
}

impl FileStore {
    pub fn new(rules_directory: PathBuf) -> Self {
        Self { rules_directory }
    }

    fn rule_path(&self, id: &str) -> PathBuf {
        self.rules_directory.join(format!("{}.urf.yaml", id))
    }
    
    pub fn get_rule_path(&self, id: &str) -> PathBuf {
        self.rule_path(id)
    }
}

impl RuleStore for FileStore {
    fn load_rule(&self, id: &str) -> Result<Option<UniversalRule>> {
        let path = self.rule_path(id);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read rule file: {}", path.display()))?;
        
        let rule: UniversalRule = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse rule file: {}", path.display()))?;
        
        Ok(Some(rule))
    }

    fn save_rule(&self, rule: &UniversalRule) -> Result<()> {
        fs::create_dir_all(&self.rules_directory)
            .with_context(|| format!("Failed to create rules directory: {}", self.rules_directory.display()))?;

        let path = self.rule_path(&rule.id);
        let content = serde_yaml::to_string(rule)
            .with_context(|| "Failed to serialize rule to YAML")?;
        
        fs::write(&path, content)
            .with_context(|| format!("Failed to write rule file: {}", path.display()))?;
        
        Ok(())
    }

    fn list_rules(&self) -> Result<Vec<String>> {
        if !self.rules_directory.exists() {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();
        for entry in fs::read_dir(&self.rules_directory)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(stem) = path.file_stem() {
                    if let Some(name) = stem.to_str() {
                        if name.ends_with(".urf") {
                            let rule_id = name.trim_end_matches(".urf");
                            rules.push(rule_id.to_string());
                        }
                    }
                }
            }
        }
        
        rules.sort();
        Ok(rules)
    }

    fn delete_rule(&self, id: &str) -> Result<()> {
        let path = self.rule_path(id);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete rule file: {}", path.display()))?;
        }
        Ok(())
    }
} 