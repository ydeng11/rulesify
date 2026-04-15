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
            context_size: 2400,
            domain: "development".to_string(),
            last_updated: "2026-04-10".to_string(),
            tags: vec!["testing".to_string()],
            install_action: Some(InstallAction::Copy {
                path: "tdd/SKILL.md".to_string(),
            }),
            score: Some(85.0),
        };
        assert_eq!(skill.stars, 1500);
        assert!(skill.install_action.unwrap().is_simple());
    }
}
