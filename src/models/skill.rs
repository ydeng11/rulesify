use crate::models::InstallAction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub source_url: String,
    pub stars: u32,
    #[serde(default)]
    pub commit_sha: String,
    #[serde(default)]
    pub context_size: u32,
    #[serde(default)]
    pub domain: String,
    pub last_updated: String,
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_action: Option<InstallAction>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

impl Skill {
    pub fn matches_tags(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }

    pub fn matches_domain(&self, domain: &str) -> bool {
        self.domain == domain
    }
}
