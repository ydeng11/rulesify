#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, Skill};
    use crate::registry::RegistryGenerator;
    use std::collections::HashMap;

    fn make_skill(id: &str) -> Skill {
        Skill {
            name: id.into(),
            description: format!("{} description for testing purposes", id),
            source_url: format!("https://github.com/test/skills/tree/main/{}", id),
            stars: 100,
            context_size: 500,
            domain: "test".into(),
            last_updated: "2026-04-10".into(),
            tags: vec!["test".into()],
            install_action: Some(InstallAction::Copy {
                folder: id.to_string(),
            }),
            score: Some(80.0),
        }
    }

    #[test]
    fn test_generate_registry() {
        let gen = RegistryGenerator::new(1);
        let mut skills = HashMap::new();
        skills.insert("tdd".into(), make_skill("tdd"));
        skills.insert("debug".into(), make_skill("debug"));

        let registry = gen.generate(skills);
        assert_eq!(registry.skills.len(), 2);
    }

    #[test]
    fn test_toml_output() {
        let gen = RegistryGenerator::new(1);
        let mut skills = HashMap::new();
        skills.insert("test".into(), make_skill("test"));

        let registry = gen.generate(skills);
        let toml = gen.to_toml(&registry);
        assert!(toml.contains("version = 1"));
        assert!(toml.contains("[skills.test]"));
    }
}
