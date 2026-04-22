use crate::models::{InstallAction, Skill};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub source_repo: String,
    pub source_folder: String,
    pub source_url: String,
    pub commit_sha: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub stars: u32,
    #[serde(default)]
    pub context_size: u32,
    #[serde(default)]
    pub domain: String,
    pub last_updated: String,
    pub install_action: InstallAction,
    #[serde(default)]
    pub is_mega_skill: bool,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

impl SkillMetadata {
    pub fn to_skill(&self, score: f32) -> Skill {
        Skill {
            name: self.name.clone(),
            description: self.description.clone(),
            source_url: self.source_url.clone(),
            stars: self.stars,
            commit_sha: self.commit_sha.clone(),
            context_size: self.context_size,
            domain: self.domain.clone(),
            last_updated: self.last_updated.clone(),
            tags: self.tags.clone(),
            install_action: Some(self.install_action.clone()),
            score: Some(score),
            is_mega_skill: self.is_mega_skill,
            dependencies: self.dependencies.clone(),
        }
    }
}
