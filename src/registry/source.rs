use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceRepo {
    AnthropicSkills,
    OpenAISkillsCurated,
    OpenAISkillsSystem,
    OpenAISkillsExperimental,
    MattPocockSkills,
    MiniMaxSkills,
    ObraSuperpowers,
    ObraSuperpowersLab,
    GsdSkills,
    PbakausImpeccable,
}

impl SourceRepo {
    pub fn all() -> Vec<Self> {
        vec![
            SourceRepo::AnthropicSkills,
            SourceRepo::OpenAISkillsCurated,
            SourceRepo::OpenAISkillsSystem,
            SourceRepo::OpenAISkillsExperimental,
            SourceRepo::MattPocockSkills,
            SourceRepo::MiniMaxSkills,
            SourceRepo::ObraSuperpowers,
            SourceRepo::ObraSuperpowersLab,
            SourceRepo::GsdSkills,
            SourceRepo::PbakausImpeccable,
        ]
    }

    pub fn owner(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "anthropics",
            SourceRepo::OpenAISkillsCurated
            | SourceRepo::OpenAISkillsSystem
            | SourceRepo::OpenAISkillsExperimental => "openai",
            SourceRepo::MattPocockSkills => "mattpocock",
            SourceRepo::MiniMaxSkills => "MiniMax-AI",
            SourceRepo::ObraSuperpowers | SourceRepo::ObraSuperpowersLab => "obra",
            SourceRepo::GsdSkills => "gsd-build",
            SourceRepo::PbakausImpeccable => "pbakaus",
        }
    }

    pub fn repo(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "skills",
            SourceRepo::OpenAISkillsCurated
            | SourceRepo::OpenAISkillsSystem
            | SourceRepo::OpenAISkillsExperimental => "skills",
            SourceRepo::MattPocockSkills => "skills",
            SourceRepo::MiniMaxSkills => "skills",
            SourceRepo::ObraSuperpowers => "superpowers",
            SourceRepo::ObraSuperpowersLab => "superpowers-lab",
            SourceRepo::GsdSkills => "get-shit-done",
            SourceRepo::PbakausImpeccable => "impeccable",
        }
    }

    pub fn branch(&self) -> &'static str {
        "main"
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner(), self.repo())
    }

    pub fn skill_pattern(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "skills/*/SKILL.md",
            SourceRepo::OpenAISkillsCurated => "skills/.curated/*/SKILL.md",
            SourceRepo::OpenAISkillsSystem => "skills/.system/*/SKILL.md",
            SourceRepo::OpenAISkillsExperimental => "skills/.experimental/*/SKILL.md",
            SourceRepo::MattPocockSkills => "*/SKILL.md",
            SourceRepo::MiniMaxSkills => "skills/*/SKILL.md",
            SourceRepo::ObraSuperpowers | SourceRepo::ObraSuperpowersLab => "",
            SourceRepo::GsdSkills => "",
            SourceRepo::PbakausImpeccable => "source/skills/impeccable/SKILL.md",
        }
    }

    pub fn is_mega_skill_collection(&self) -> bool {
        matches!(
            self,
            SourceRepo::ObraSuperpowers
                | SourceRepo::ObraSuperpowersLab
                | SourceRepo::GsdSkills
                | SourceRepo::PbakausImpeccable
        )
    }

    pub fn parse_skill_id(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();

        match self {
            SourceRepo::AnthropicSkills | SourceRepo::MiniMaxSkills => {
                if parts.len() >= 3 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[1].to_string())
                } else {
                    None
                }
            }
            SourceRepo::OpenAISkillsCurated
            | SourceRepo::OpenAISkillsSystem
            | SourceRepo::OpenAISkillsExperimental => {
                if parts.len() >= 4 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            }
            SourceRepo::MattPocockSkills => {
                if parts.len() >= 2 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            }
            SourceRepo::ObraSuperpowers
            | SourceRepo::ObraSuperpowersLab
            | SourceRepo::GsdSkills => None,
            SourceRepo::PbakausImpeccable => {
                if path == "source/skills/impeccable/SKILL.md" {
                    Some("impeccable".to_string())
                } else {
                    None
                }
            }
        }
    }

    pub fn matches_pattern(&self, path: &str) -> bool {
        path.ends_with("SKILL.md") && self.parse_skill_id(path).is_some()
    }

    pub fn parse_skill_folder(&self, path: &str) -> Option<String> {
        if !path.ends_with("SKILL.md") {
            return None;
        }
        Some(path.replace("/SKILL.md", ""))
    }

    pub fn mega_skill_source_folder(&self) -> &'static str {
        match self {
            SourceRepo::ObraSuperpowers => "skills",
            SourceRepo::ObraSuperpowersLab => "skills",
            SourceRepo::GsdSkills => "",
            SourceRepo::PbakausImpeccable => "source/skills",
            _ => "",
        }
    }

    pub fn mega_skill_dest_name(&self) -> &'static str {
        match self {
            SourceRepo::ObraSuperpowers => "superpowers",
            SourceRepo::ObraSuperpowersLab => "superpowers-lab",
            SourceRepo::GsdSkills => "gsd",
            SourceRepo::PbakausImpeccable => "impeccable",
            _ => "",
        }
    }
}
