use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InstallAction {
    #[serde(rename = "copy")]
    Copy { path: String },
    #[serde(rename = "command")]
    Command { value: String },
}

impl InstallAction {
    pub fn is_simple(&self) -> bool {
        matches!(self, InstallAction::Copy { .. })
    }

    pub fn install_command(&self, source_url: &str) -> Option<String> {
        match self {
            InstallAction::Copy { path } => {
                let file_url = format!("{}/{}", source_url.replace("/tree/", "/blob/"), path);
                Some(format!("rulesify skill fetch {}", file_url))
            }
            InstallAction::Command { value } => Some(value.clone()),
        }
    }

    pub fn default_copy(skill_path: &str) -> Self {
        InstallAction::Copy {
            path: skill_path.to_string(),
        }
    }
}
