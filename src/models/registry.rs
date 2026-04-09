use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::Skill;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub updated: String,
    pub skills: HashMap<String, Skill>,
}

impl Registry {
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }
    
    pub fn filter_by_tools(&self, tools: &[String]) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_tools(tools))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_tags(tags))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn all_skills(&self) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}