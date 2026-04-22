#[cfg(test)]
mod tests {
    use crate::models::{InstalledSkill, ProjectConfig, Scope};

    #[test]
    fn test_scope_default() {
        let scope = Scope::default();
        assert_eq!(scope, Scope::Project);
    }

    #[test]
    fn test_scope_serialization() {
        let project = Scope::Project;
        let global = Scope::Global;

        let project_str = serde_json::to_string(&project).unwrap();
        let global_str = serde_json::to_string(&global).unwrap();

        assert_eq!(project_str, "\"project\"");
        assert_eq!(global_str, "\"global\"");
    }

    #[test]
    fn test_scope_deserialization() {
        let project: Scope = serde_json::from_str("\"project\"").unwrap();
        let global: Scope = serde_json::from_str("\"global\"").unwrap();

        assert_eq!(project, Scope::Project);
        assert_eq!(global, Scope::Global);
    }

    #[test]
    fn test_installed_skill_with_scope() {
        let skill = InstalledSkill {
            added: "2026-04-16".to_string(),
            source: "https://example.com".to_string(),
            commit_sha: "abc123".to_string(),
            scope: Scope::Global,
        };

        let toml = toml::to_string_pretty(&skill).unwrap();
        assert!(toml.contains("scope = \"global\""));
    }

    #[test]
    fn test_config_add_skill_with_scope() {
        let mut config = ProjectConfig::new();
        config.add_skill("my-skill", "https://example.com", "abc123", Scope::Project);
        config.add_skill(
            "global-skill",
            "https://example.com",
            "def456",
            Scope::Global,
        );

        assert_eq!(config.installed_skills.len(), 2);
        assert_eq!(config.installed_skills["my-skill"].scope, Scope::Project);
        assert_eq!(config.installed_skills["global-skill"].scope, Scope::Global);
    }

    #[test]
    fn test_config_serialization_with_scope() {
        let mut config = ProjectConfig::new();
        config.add_skill("project-skill", "https://...", "abc123", Scope::Project);
        config.add_skill("global-skill", "https://...", "def456", Scope::Global);

        let toml = toml::to_string_pretty(&config).unwrap();
        assert!(toml.contains("scope = \"project\""));
        assert!(toml.contains("scope = \"global\""));

        let parsed: ProjectConfig = toml::from_str(&toml).unwrap();
        assert_eq!(
            parsed.installed_skills["project-skill"].scope,
            Scope::Project
        );
        assert_eq!(parsed.installed_skills["global-skill"].scope, Scope::Global);
    }

    #[test]
    fn test_config_backward_compatibility() {
        let toml_str = r#"
version = 1
tools = []
[installed_skills.old-skill]
added = "2026-04-16"
source = "https://example.com"
commit_sha = "legacy123"
"#;

        let config: ProjectConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.installed_skills["old-skill"].scope, Scope::Project);
    }
}
