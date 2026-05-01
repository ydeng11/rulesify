use crate::utils::{reconcile_project_config, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    #[default]
    Project,
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub added: String,
    pub source: String,
    pub commit_sha: String,
    #[serde(default)]
    pub scope: Scope,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: u32,
    pub tools: Vec<String>,
    pub installed_skills: HashMap<String, InstalledSkill>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            version: 1,
            tools: Vec::new(),
            installed_skills: HashMap::new(),
        }
    }
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_skill(&mut self, id: &str, source: &str, commit_sha: &str, scope: Scope) {
        self.installed_skills.insert(
            id.to_string(),
            InstalledSkill {
                added: chrono::Local::now().format("%Y-%m-%d").to_string(),
                source: source.to_string(),
                commit_sha: commit_sha.to_string(),
                scope,
            },
        );
    }

    pub fn update_skill_sha(&mut self, id: &str, commit_sha: &str) {
        if let Some(skill) = self.installed_skills.get_mut(id) {
            skill.commit_sha = commit_sha.to_string();
        }
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

    pub fn reconcile_and_load(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;
        let mut config: ProjectConfig = toml::from_str(&content)?;

        reconcile_project_config(&mut config);

        if config.installed_skills.is_empty() {
            if let Err(e) = std::fs::remove_file(path) {
                log::error!("Failed to remove empty config file: {}", e);
            }
            return Ok(None);
        }

        if let Err(e) = std::fs::write(path, toml::to_string_pretty(&config)?) {
            log::error!("Failed to save reconciled project config: {}", e);
        }

        Ok(Some(config))
    }
}
