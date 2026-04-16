#[cfg(test)]
mod tests {
    use crate::installer::instructions::{
        generate_install_instructions, generate_uninstall_instructions,
    };
    use crate::models::Scope;

    #[test]
    fn test_install_instructions_project_scope() {
        let instructions = generate_install_instructions(
            "my-skill",
            "https://github.com/example/skills/my-skill",
            &["cursor".to_string(), "claude-code".to_string()],
            Scope::Project,
        );

        assert!(instructions.contains("# Install Instructions"));
        assert!(instructions.contains("Scope: project level"));
        assert!(instructions.contains(".cursor/skills/my-skill/SKILL.md"));
        assert!(instructions.contains(".claude/skills/my-skill/SKILL.md"));
        assert!(instructions.contains("https://github.com/example/skills/my-skill"));
    }

    #[test]
    fn test_install_instructions_global_scope() {
        let instructions = generate_install_instructions(
            "my-skill",
            "https://github.com/example/skills/my-skill",
            &["cursor".to_string(), "claude-code".to_string()],
            Scope::Global,
        );

        assert!(instructions.contains("# Install Instructions"));
        assert!(instructions.contains("Scope: global level"));
        assert!(instructions.contains("cursor/skills/my-skill/SKILL.md"));
        assert!(instructions.contains("claude/skills/my-skill/SKILL.md"));
    }

    #[test]
    fn test_uninstall_instructions_project_scope() {
        let instructions = generate_uninstall_instructions(
            "my-skill",
            &["cursor".to_string(), "claude-code".to_string()],
            Scope::Project,
        );

        assert!(instructions.contains("# Uninstall Instructions"));
        assert!(instructions.contains("Scope: project level"));
        assert!(instructions.contains(".cursor/skills/my-skill"));
        assert!(instructions.contains(".claude/skills/my-skill"));
        assert!(instructions.contains("Delete folder"));
    }

    #[test]
    fn test_uninstall_instructions_global_scope() {
        let instructions = generate_uninstall_instructions(
            "my-skill",
            &["cursor".to_string(), "claude-code".to_string()],
            Scope::Global,
        );

        assert!(instructions.contains("# Uninstall Instructions"));
        assert!(instructions.contains("Scope: global level"));
        assert!(instructions.contains("Delete folder"));
    }

    #[test]
    fn test_install_instructions_all_tools() {
        let instructions = generate_install_instructions(
            "test-skill",
            "https://example.com",
            &[
                "claude-code".to_string(),
                "codex".to_string(),
                "cursor".to_string(),
                "opencode".to_string(),
                "pi".to_string(),
            ],
            Scope::Project,
        );

        assert!(instructions.contains("claude-code"));
        assert!(instructions.contains("codex"));
        assert!(instructions.contains("cursor"));
        assert!(instructions.contains("opencode"));
        assert!(instructions.contains("pi"));
        assert!(instructions.contains(".claude/skills"));
        assert!(instructions.contains(".agents/skills"));
        assert!(instructions.contains(".cursor/skills"));
        assert!(instructions.contains(".opencode/skills"));
        assert!(instructions.contains(".pi/skills/pi-skills"));
    }
}
