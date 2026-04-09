use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub added: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: u32,
    pub tools: Vec<String>,
    pub installed_skills: HashMap<String, InstalledSkill>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self {
            version: 1,
            tools: Vec::new(),
            installed_skills: HashMap::new(),
        }
    }
    
    pub fn add_skill(&mut self, id: &str, source: &str) {
        self.installed_skills.insert(
            id.to_string(),
            InstalledSkill {
                added: chrono::Local::now().format("%Y-%m-%d").to_string(),
                source: source.to_string(),
            },
        );
    }
    
    pub fn remove_skill(&mut self, id: &str) -> Option<InstalledSkill> {
        self.installed_skills.remove(id)
    }
    
    pub fn list_skills(&self) -> Vec<(String, InstalledSkill)> {
        self.installed_skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}