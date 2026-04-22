use crate::models::Scope;
use std::path::PathBuf;

pub fn get_skill_path(tool: &str, scope: Scope, skill_name: &str) -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"));

    match tool {
        "claude-code" => match scope {
            Scope::Project => PathBuf::from(".claude/skills")
                .join(skill_name)
                .join("SKILL.md"),
            Scope::Global => home
                .join(".claude/skills")
                .join(skill_name)
                .join("SKILL.md"),
        },
        "codex" => match scope {
            Scope::Project => PathBuf::from(".agents/skills")
                .join(skill_name)
                .join("SKILL.md"),
            Scope::Global => home
                .join(".agents/skills")
                .join(skill_name)
                .join("SKILL.md"),
        },
        "cursor" => match scope {
            Scope::Project => PathBuf::from(".cursor/skills")
                .join(skill_name)
                .join("SKILL.md"),
            Scope::Global => home
                .join(".cursor/skills")
                .join(skill_name)
                .join("SKILL.md"),
        },
        "opencode" => match scope {
            Scope::Project => PathBuf::from(".opencode/skills")
                .join(skill_name)
                .join("SKILL.md"),
            Scope::Global => home
                .join(".config/opencode/skills")
                .join(skill_name)
                .join("SKILL.md"),
        },
        "pi" => match scope {
            Scope::Project => PathBuf::from(".pi/skills/pi-skills")
                .join(skill_name)
                .join("SKILL.md"),
            Scope::Global => home
                .join(".pi/agent/skills/pi-skills")
                .join(skill_name)
                .join("SKILL.md"),
        },
        _ => PathBuf::from(".agents/skills")
            .join(skill_name)
            .join("SKILL.md"),
    }
}

pub fn get_skill_folder(tool: &str, scope: Scope, skill_name: &str) -> PathBuf {
    get_skill_path(tool, scope, skill_name)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
}
