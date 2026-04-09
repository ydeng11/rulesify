use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub source: String,
    pub tags: Vec<String>,
    pub compatible_tools: Vec<String>,
    pub popularity: u32,
}

impl Skill {
    pub fn matches_tools(&self, tools: &[String]) -> bool {
        tools.iter().any(|t| self.compatible_tools.contains(t))
    }
    
    pub fn matches_tags(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }
}