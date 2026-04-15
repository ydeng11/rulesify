use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InstallAction {
    #[serde(rename = "copy")]
    Copy { folder: String },
    #[serde(rename = "command")]
    Command { value: String },
}

impl InstallAction {
    pub fn is_simple(&self) -> bool {
        matches!(self, InstallAction::Copy { .. })
    }

    pub fn default_copy(folder: &str) -> Self {
        InstallAction::Copy {
            folder: folder.to_string(),
        }
    }
}
