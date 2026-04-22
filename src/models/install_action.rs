use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InstallAction {
    #[serde(rename = "copy")]
    Copy { folder: String },
    #[serde(rename = "command")]
    Command { value: String },
    #[serde(rename = "npx")]
    Npx {
        package: String,
        args: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        uninstall_flag: Option<String>,
    },
    #[serde(rename = "mega-skill-copy")]
    MegaSkillCopy {
        source_folder: String,
        dest_name: String,
    },
}

impl InstallAction {
    pub fn is_simple(&self) -> bool {
        matches!(self, InstallAction::Copy { .. })
    }

    pub fn is_npx(&self) -> bool {
        matches!(self, InstallAction::Npx { .. })
    }

    pub fn is_mega_skill_copy(&self) -> bool {
        matches!(self, InstallAction::MegaSkillCopy { .. })
    }

    pub fn default_copy(folder: &str) -> Self {
        InstallAction::Copy {
            folder: folder.to_string(),
        }
    }

    pub fn mega_skill_copy(source_folder: &str, dest_name: &str) -> Self {
        InstallAction::MegaSkillCopy {
            source_folder: source_folder.to_string(),
            dest_name: dest_name.to_string(),
        }
    }
}
