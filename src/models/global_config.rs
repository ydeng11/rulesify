use crate::models::{InstalledSkill, Scope};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub fn get_global_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("rulesify")
        .join(".registry.toml")
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub version: u32,
    pub installed_skills: HashMap<String, HashMap<String, InstalledSkill>>,
}

impl GlobalConfig {
    pub fn new() -> Self {
        Self {
            version: 1,
            installed_skills: HashMap::new(),
        }
    }

    pub fn load() -> Self {
        let path = get_global_config_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(mut config) = toml::from_str(&content) {
                    crate::utils::reconcile_global_config(&mut config);
                    if !config.installed_skills.is_empty() {
                        if let Err(e) = config.save() {
                            log::error!("Failed to save reconciled global config: {}", e);
                        }
                    }
                    return config;
                }
            }
        }
        Self::new()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let path = get_global_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, content)
    }

    pub fn add_skill(&mut self, tool: &str, id: &str, source: &str, commit_sha: &str) {
        let tool_skills = self.installed_skills.entry(tool.to_string()).or_default();
        tool_skills.insert(
            id.to_string(),
            InstalledSkill {
                added: chrono::Local::now().format("%Y-%m-%d").to_string(),
                source: source.to_string(),
                commit_sha: commit_sha.to_string(),
                scope: Scope::Global,
            },
        );
    }

    pub fn remove_skill(&mut self, tool: &str, id: &str) -> Option<InstalledSkill> {
        if let Some(tool_skills) = self.installed_skills.get_mut(tool) {
            tool_skills.remove(id)
        } else {
            None
        }
    }

    pub fn get_skill_for_tool(&self, tool: &str, id: &str) -> Option<&InstalledSkill> {
        self.installed_skills.get(tool).and_then(|m| m.get(id))
    }

    pub fn is_skill_installed_for_tool(&self, tool: &str, id: &str) -> bool {
        self.get_skill_for_tool(tool, id).is_some()
    }

    pub fn is_skill_installed_globally(&self, id: &str) -> bool {
        self.installed_skills.values().any(|m| m.contains_key(id))
    }

    pub fn get_tools_for_skill(&self, id: &str) -> Vec<String> {
        self.installed_skills
            .iter()
            .filter(|(_, skills)| skills.contains_key(id))
            .map(|(tool, _)| tool.clone())
            .collect()
    }

    pub fn list_all_skills(&self) -> Vec<(String, String, InstalledSkill)> {
        self.installed_skills
            .iter()
            .flat_map(|(tool, skills)| {
                skills
                    .iter()
                    .map(|(id, info)| (tool.clone(), id.clone(), info.clone()))
            })
            .collect()
    }

    pub fn list_skills_for_tool(&self, tool: &str) -> Vec<(String, InstalledSkill)> {
        self.installed_skills
            .get(tool)
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default()
    }

    pub fn update_skill_sha(&mut self, tool: &str, id: &str, commit_sha: &str) {
        if let Some(tool_skills) = self.installed_skills.get_mut(tool) {
            if let Some(skill) = tool_skills.get_mut(id) {
                skill.commit_sha = commit_sha.to_string();
            }
        }
    }
}
