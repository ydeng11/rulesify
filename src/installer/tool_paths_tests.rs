#[cfg(test)]
mod tests {
    use crate::installer::tool_paths::{get_skill_folder, get_skill_path};
    use crate::models::Scope;
    use std::path::PathBuf;

    #[test]
    fn test_project_skill_paths() {
        let skill = "test-driven-development";

        assert_eq!(
            get_skill_path("claude-code", Scope::Project, skill),
            PathBuf::from(".claude/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("codex", Scope::Project, skill),
            PathBuf::from(".agents/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("cursor", Scope::Project, skill),
            PathBuf::from(".cursor/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("opencode", Scope::Project, skill),
            PathBuf::from(".opencode/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("pi", Scope::Project, skill),
            PathBuf::from(".pi/skills/pi-skills/test-driven-development/SKILL.md")
        );
    }

    #[test]
    fn test_global_skill_paths() {
        let skill = "test-driven-development";
        let home = dirs::home_dir().unwrap();

        assert_eq!(
            get_skill_path("claude-code", Scope::Global, skill),
            home.join(".claude/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("codex", Scope::Global, skill),
            home.join(".agents/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("cursor", Scope::Global, skill),
            home.join(".cursor/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("opencode", Scope::Global, skill),
            home.join(".config/opencode/skills/test-driven-development/SKILL.md")
        );
        assert_eq!(
            get_skill_path("pi", Scope::Global, skill),
            home.join(".pi/agent/skills/pi-skills/test-driven-development/SKILL.md")
        );
    }

    #[test]
    fn test_skill_folder_project() {
        let skill = "my-skill";

        assert_eq!(
            get_skill_folder("claude-code", Scope::Project, skill),
            PathBuf::from(".claude/skills/my-skill")
        );
        assert_eq!(
            get_skill_folder("cursor", Scope::Project, skill),
            PathBuf::from(".cursor/skills/my-skill")
        );
    }

    #[test]
    fn test_skill_folder_global() {
        let skill = "my-skill";
        let home = dirs::home_dir().unwrap();

        assert_eq!(
            get_skill_folder("claude-code", Scope::Global, skill),
            home.join(".claude/skills/my-skill")
        );
        assert_eq!(
            get_skill_folder("cursor", Scope::Global, skill),
            home.join(".cursor/skills/my-skill")
        );
    }

    #[test]
    fn test_unknown_tool_fallback() {
        let path = get_skill_path("unknown-tool", Scope::Project, "my-skill");
        assert_eq!(path, PathBuf::from(".agents/skills/my-skill/SKILL.md"));
    }
}
