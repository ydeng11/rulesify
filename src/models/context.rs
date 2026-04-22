use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub existing_tools: Vec<String>,
}

impl ProjectContext {
    pub fn to_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        tags.extend(self.languages.iter().cloned());
        tags.extend(self.frameworks.iter().cloned());
        tags
    }

    pub fn has_tool(&self, tool: &str) -> bool {
        self.existing_tools.contains(&tool.to_string())
    }
}
