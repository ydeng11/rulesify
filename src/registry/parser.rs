use crate::utils::{Result, RulesifyError};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct SkillParser;

impl SkillParser {
    pub fn parse(content: &str) -> Result<ParsedSkill> {
        let frontmatter = Self::extract_frontmatter(content)?;
        let parsed: ParsedSkill = serde_yaml::from_str(&frontmatter)
            .map_err(|e| RulesifyError::SkillParse(format!("YAML error: {}", e)))?;

        Self::validate(&parsed)?;

        Ok(parsed)
    }

    fn extract_frontmatter(content: &str) -> Result<String> {
        if !content.starts_with("---") {
            return Err(RulesifyError::SkillParse("Missing frontmatter".into()).into());
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut end_idx = None;

        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                end_idx = Some(i);
                break;
            }
        }

        if end_idx.is_none() {
            return Err(RulesifyError::SkillParse("Unclosed frontmatter".into()).into());
        }

        Ok(lines[1..end_idx.unwrap()].join("\n"))
    }

    fn validate(parsed: &ParsedSkill) -> Result<()> {
        if parsed.name.trim().is_empty() {
            return Err(RulesifyError::SkillParse("name required".into()).into());
        }
        if parsed.description.len() < 20 {
            return Err(
                RulesifyError::SkillParse("description must be at least 20 chars".into()).into(),
            );
        }
        Ok(())
    }

    pub fn estimate_context_size(content: &str) -> u32 {
        let chars = content.len();
        let tokens = chars / 4;
        tokens as u32
    }
}
