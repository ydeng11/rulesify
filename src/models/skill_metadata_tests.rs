#[cfg(test)]
mod tests {
    use crate::models::InstallAction;
    use crate::models::SkillMetadata;

    #[test]
    fn test_metadata_creation() {
        let meta = SkillMetadata {
            skill_id: "tdd".to_string(),
            name: "Test-Driven Development".to_string(),
            description: "Write tests before implementation".to_string(),
            source_repo: "mattpoclock/skills".to_string(),
            source_folder: "tdd/SKILL.md".to_string(),
            source_url: "https://github.com/mattpoclock/skills/tree/main/tdd".to_string(),
            commit_sha: "abc123def".to_string(),
            tags: vec!["testing".to_string()],
            stars: 1500,
            context_size: 2400,
            domain: "development".to_string(),
            last_updated: "2026-04-10".to_string(),
            install_action: InstallAction::Copy {
                folder: "tdd".to_string(),
            },
            is_mega_skill: false,
            dependencies: Vec::new(),
        };
        assert_eq!(meta.skill_id, "tdd");
        assert!(meta.install_action.is_simple());
    }
}
