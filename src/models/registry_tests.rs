#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, Registry, Skill};
    use std::collections::HashMap;

    #[test]
    fn test_registry_new_format() {
        let mut skills = HashMap::new();
        skills.insert(
            "tdd".to_string(),
            Skill {
                name: "TDD".to_string(),
                description: "Test driven development methodology".to_string(),
                source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
                stars: 1500,
                context_size: 2400,
                domain: "development".to_string(),
                last_updated: "2026-04-10".to_string(),
                tags: vec!["testing".to_string()],
                install_action: Some(InstallAction::Copy {
                    folder: "tdd".to_string(),
                }),
                score: Some(85.0),
            },
        );

        let registry = Registry {
            version: 1,
            updated: "2026-04-14".to_string(),
            skills,
        };

        assert_eq!(registry.skills.len(), 1);
        assert!(registry.get_skill("tdd").is_some());
    }
}
