#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, Skill};

    #[test]
    fn test_skill_with_new_fields() {
        let skill = Skill {
            name: "TDD".to_string(),
            description: "Test driven development".to_string(),
            source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
            stars: 1500,
            commit_sha: "abc123def".to_string(),
            context_size: 2400,
            domain: "development".to_string(),
            last_updated: "2026-04-10".to_string(),
            tags: vec!["testing".to_string()],
            install_action: Some(InstallAction::Copy {
                folder: "tdd".to_string(),
            }),
            score: Some(85.0),
            is_mega_skill: false,
        };
        assert_eq!(skill.stars, 1500);
        assert!(skill.install_action.unwrap().is_simple());
        assert!(!skill.is_mega_skill);
    }

    #[test]
    fn test_mega_skill() {
        let skill = Skill {
            name: "superpowers".to_string(),
            description: "Complete software development methodology for coding agents".to_string(),
            source_url: "https://github.com/obra/superpowers/tree/main/skills".to_string(),
            stars: 160000,
            commit_sha: String::new(),
            context_size: 0,
            domain: "development".to_string(),
            last_updated: "2026-04-20".to_string(),
            tags: vec!["mega-skill".to_string(), "workflow".to_string()],
            install_action: Some(InstallAction::mega_skill_copy("skills", "superpowers")),
            score: Some(100.0),
            is_mega_skill: true,
        };
        assert!(skill.is_mega_skill);
        assert!(skill.install_action.unwrap().is_mega_skill_copy());
    }

    #[test]
    fn test_mega_skill_serialization() {
        let skill = Skill {
            name: "gsd".to_string(),
            description: "Get Shit Done project management system".to_string(),
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
        };

        let serialized = serde_json::to_string(&skill).unwrap();
        assert!(serialized.contains("is_mega_skill"));
        assert!(serialized.contains("true"));
        assert!(serialized.contains("npx"));

        let deserialized: Skill = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.is_mega_skill);
    }
}
