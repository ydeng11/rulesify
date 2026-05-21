use crate::models::Scope;
use std::path::PathBuf;

fn skills_base_path(tool: &str, scope: Scope) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    match tool {
        "claude-code" => match scope {
            Scope::Project => PathBuf::from(".claude/skills"),
            Scope::Global => home.join(".claude/skills"),
        },
        "codex" => match scope {
            Scope::Project => PathBuf::from(".agents/skills"),
            Scope::Global => home.join(".agents/skills"),
        },
        "cursor" => match scope {
            Scope::Project => PathBuf::from(".cursor/skills"),
            Scope::Global => home.join(".cursor/skills"),
        },
        "opencode" => match scope {
            Scope::Project => PathBuf::from(".opencode/skills"),
            Scope::Global => home.join(".config/opencode/skills"),
        },
        "pi" => match scope {
            Scope::Project => PathBuf::from(".pi/skills/pi-skills"),
            Scope::Global => home.join(".pi/agent/skills/pi-skills"),
        },
        _ => match scope {
            Scope::Project => PathBuf::from(".agents/skills"),
            Scope::Global => home.join(".agents/skills"),
        },
    }
}

pub fn get_skill_path(tool: &str, scope: Scope, skill_name: &str) -> PathBuf {
    skills_base_path(tool, scope)
        .join(skill_name)
        .join("SKILL.md")
}

pub fn get_skill_folder(tool: &str, scope: Scope, skill_name: &str) -> PathBuf {
    skills_base_path(tool, scope).join(skill_name)
}

/// Returns the directory containing all installed skills for a tool (project scope).
pub fn get_skills_parent_dir(tool: &str) -> PathBuf {
    skills_base_path(tool, Scope::Project)
}
