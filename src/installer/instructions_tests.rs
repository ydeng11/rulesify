#[cfg(test)]
mod tests {
    use crate::installer::instructions::{
        generate_install_instructions, generate_instructions, generate_uninstall_instructions,
    };
    use crate::models::{InstallAction, Scope, Skill};

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

    #[test]
    fn test_mega_skill_copy_instructions() {
        let skill = Skill {
            name: "superpowers".to_string(),
            description: "Complete software development methodology".to_string(),
            source_url: "https://github.com/obra/superpowers/tree/main/skills".to_string(),
            stars: 160000,
            commit_sha: String::new(),
            context_size: 0,
            domain: "development".to_string(),
            last_updated: "2026-04-20".to_string(),
            tags: vec!["mega-skill".to_string()],
            install_action: Some(InstallAction::mega_skill_copy("skills", "superpowers")),
            score: Some(100.0),
            is_mega_skill: true,
            dependencies: Vec::new(),
        };

        let instructions = generate_instructions(
            &[("superpowers".to_string(), skill)],
            &["claude-code".to_string()],
        );

        assert!(instructions.contains("# Installation Instructions"));
        assert!(instructions.contains("superpowers"));
        assert!(instructions.contains("Mega-skill"));
        assert!(instructions.contains("skills"));
    }

    #[test]
    fn test_npx_install_instructions() {
        let skill = Skill {
            name: "gsd".to_string(),
            description: "Get Shit Done project management".to_string(),
            source_url: "https://github.com/gsd-build/get-shit-done".to_string(),
            stars: 200,
            commit_sha: String::new(),
            context_size: 0,
            domain: "development".to_string(),
            last_updated: "2026-04-20".to_string(),
            tags: vec!["mega-skill".to_string()],
            install_action: Some(InstallAction::Npx {
                package: "get-shit-done-cc".to_string(),
                args: vec!["@latest".to_string()],
                uninstall_flag: None,
            }),
            score: Some(85.0),
            is_mega_skill: true,
            dependencies: Vec::new(),
        };

        let instructions =
            generate_instructions(&[("gsd".to_string(), skill)], &["claude-code".to_string()]);

        assert!(instructions.contains("npx"));
        assert!(instructions.contains("get-shit-done-cc"));
        assert!(instructions.contains("@latest"));
    }

    #[test]
    fn test_mega_skill_marker_in_instructions() {
        let skill = Skill {
            name: "impeccable".to_string(),
            description: "The design language for AI harnesses".to_string(),
            source_url: "https://github.com/pbakaus/impeccable/tree/main/source/skills/impeccable"
                .to_string(),
            stars: 20700,
            commit_sha: String::new(),
            context_size: 0,
            domain: "design-and-media".to_string(),
            last_updated: "2026-04-20".to_string(),
            tags: vec!["mega-skill".to_string()],
            install_action: Some(InstallAction::mega_skill_copy(
                "source/skills",
                "impeccable",
            )),
            score: Some(90.0),
            is_mega_skill: true,
            dependencies: Vec::new(),
        };

        let instructions = generate_instructions(
            &[("impeccable".to_string(), skill)],
            &["cursor".to_string()],
        );

        assert!(instructions.contains("**Type:** Mega-skill"));
    }
}
