#[cfg(test)]
mod tests {
    use crate::models::{GlobalConfig, ProjectConfig, Scope};
    use crate::utils::{reconcile_global_config, reconcile_project_config, skill_exists_on_disk};
    use serial_test::serial;
    use std::fs;
    use tempfile::TempDir;

    fn setup_project_skill(base_dir: &std::path::Path, tool: &str, skill_name: &str) {
        let skill_folder = match tool {
            "claude-code" => base_dir.join(".claude/skills").join(skill_name),
            "codex" => base_dir.join(".agents/skills").join(skill_name),
            "cursor" => base_dir.join(".cursor/skills").join(skill_name),
            _ => base_dir.join(".agents/skills").join(skill_name),
        };

        fs::create_dir_all(&skill_folder).unwrap();
        fs::write(skill_folder.join("SKILL.md"), "# Test Skill").unwrap();
    }

    fn setup_project_skill_folder_only(base_dir: &std::path::Path, tool: &str, skill_name: &str) {
        let skill_folder = match tool {
            "claude-code" => base_dir.join(".claude/skills").join(skill_name),
            "codex" => base_dir.join(".agents/skills").join(skill_name),
            "cursor" => base_dir.join(".cursor/skills").join(skill_name),
            _ => base_dir.join(".agents/skills").join(skill_name),
        };

        fs::create_dir_all(&skill_folder).unwrap();
    }

    #[test]
    #[serial]
    fn test_skill_exists_on_disk_true() {
        let temp_dir = TempDir::new().unwrap();
        setup_project_skill(temp_dir.path(), "claude-code", "test-skill-true");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let exists = skill_exists_on_disk("claude-code", Scope::Project, "test-skill-true");

        std::env::set_current_dir(original_dir).unwrap();

        assert!(exists);
    }

    #[test]
    fn test_skill_exists_on_disk_false_no_folder() {
        let temp_dir = TempDir::new().unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let exists = skill_exists_on_disk("claude-code", Scope::Project, "nonexistent-skill");

        std::env::set_current_dir(original_dir).unwrap();

        assert!(!exists);
    }

    #[test]
    #[serial]
    fn test_skill_exists_on_disk_false_no_md() {
        let temp_dir = TempDir::new().unwrap();
        setup_project_skill_folder_only(temp_dir.path(), "claude-code", "test-skill-no-md");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let exists = skill_exists_on_disk("claude-code", Scope::Project, "test-skill-no-md");

        std::env::set_current_dir(original_dir).unwrap();

        assert!(!exists);
    }

    #[test]
    #[serial]
    fn test_reconcile_project_config_removes_all_tools_missing() {
        let mut config = ProjectConfig::new();
        config.tools = vec!["claude-code".to_string(), "codex".to_string()];

        config.add_skill(
            "missing-skill",
            "https://test.com",
            "abc123",
            Scope::Project,
        );
        config.add_skill(
            "existing-skill",
            "https://test.com",
            "def456",
            Scope::Project,
        );

        let temp_dir = TempDir::new().unwrap();
        setup_project_skill(temp_dir.path(), "claude-code", "existing-skill");
        setup_project_skill(temp_dir.path(), "codex", "existing-skill");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = reconcile_project_config(&mut config);

        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], "missing-skill".to_string());
        assert!(!config.installed_skills.contains_key("missing-skill"));
        assert!(config.installed_skills.contains_key("existing-skill"));
    }

    #[test]
    #[serial]
    fn test_reconcile_project_config_keeps_partial_tools() {
        let mut config = ProjectConfig::new();
        config.tools = vec!["claude-code".to_string(), "codex".to_string()];

        config.add_skill(
            "partial-skill",
            "https://test.com",
            "abc123",
            Scope::Project,
        );

        let temp_dir = TempDir::new().unwrap();
        setup_project_skill(temp_dir.path(), "codex", "partial-skill");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = reconcile_project_config(&mut config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(removed.is_empty());
        assert!(config.installed_skills.contains_key("partial-skill"));
    }

    #[test]
    #[serial]
    fn test_reconcile_project_config_keeps_all_when_present() {
        let mut config = ProjectConfig::new();
        config.tools = vec!["claude-code".to_string()];

        config.add_skill("skill1", "https://test.com", "abc123", Scope::Project);
        config.add_skill("skill2", "https://test.com", "def456", Scope::Project);

        let temp_dir = TempDir::new().unwrap();
        setup_project_skill(temp_dir.path(), "claude-code", "skill1");
        setup_project_skill(temp_dir.path(), "claude-code", "skill2");

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = reconcile_project_config(&mut config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(removed.is_empty());
        assert!(config.installed_skills.contains_key("skill1"));
        assert!(config.installed_skills.contains_key("skill2"));
    }

    #[test]
    #[serial]
    fn test_reconcile_mega_skill_keeps_with_main_md() {
        let mut config = ProjectConfig::new();
        config.tools = vec!["claude-code".to_string()];

        let skill_name = "mega-skill-keep-test";
        config.add_skill(skill_name, "https://test.com", "abc123", Scope::Project);

        let temp_dir = TempDir::new().unwrap();
        setup_project_skill(temp_dir.path(), "claude-code", skill_name);

        let mega_folder = temp_dir.path().join(".claude/skills").join(skill_name);
        fs::create_dir_all(mega_folder.join("commands")).unwrap();
        fs::create_dir_all(mega_folder.join("workflows")).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = reconcile_project_config(&mut config);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(removed.is_empty());
        assert!(config.installed_skills.contains_key(skill_name));
    }

    #[test]
    #[serial]
    fn test_reconcile_mega_skill_removes_without_main_md() {
        let mut config = ProjectConfig::new();
        config.tools = vec!["claude-code".to_string()];

        let skill_name = "mega-skill-remove-test";
        config.add_skill(skill_name, "https://test.com", "abc123", Scope::Project);

        let temp_dir = TempDir::new().unwrap();
        let mega_folder = temp_dir.path().join(".claude/skills").join(skill_name);
        fs::create_dir_all(&mega_folder).unwrap();
        fs::create_dir_all(mega_folder.join("commands")).unwrap();
        fs::create_dir_all(mega_folder.join("workflows")).unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let removed = reconcile_project_config(&mut config);

        std::env::set_current_dir(original_dir).unwrap();

        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0], skill_name.to_string());
        assert!(!config.installed_skills.contains_key(skill_name));
    }

    #[test]
    fn test_reconcile_global_config_removes_missing() {
        let mut config = GlobalConfig::new();

        config.add_skill(
            "claude-code",
            "missing-global",
            "https://test.com",
            "abc123",
        );
        config.add_skill(
            "claude-code",
            "existing-global",
            "https://test.com",
            "def456",
        );

        let home = dirs::home_dir().unwrap();
        let existing_folder = home.join(".claude/skills/existing-global");
        if existing_folder.exists() {
            fs::remove_dir_all(&existing_folder).unwrap();
        }
        fs::create_dir_all(&existing_folder).unwrap();
        fs::write(existing_folder.join("SKILL.md"), "# Skill").unwrap();

        let removed = reconcile_global_config(&mut config);

        if existing_folder.exists() {
            fs::remove_dir_all(&existing_folder).unwrap();
        }

        assert_eq!(removed.len(), 1);
        assert_eq!(
            removed[0],
            ("claude-code".to_string(), "missing-global".to_string())
        );
        assert!(!config.is_skill_installed_for_tool("claude-code", "missing-global"));
        assert!(config.is_skill_installed_for_tool("claude-code", "existing-global"));
    }

    #[test]
    fn test_reconcile_global_config_keeps_all_when_present() {
        let mut config = GlobalConfig::new();

        config.add_skill(
            "claude-code",
            "existing-global-1",
            "https://test.com",
            "abc123",
        );
        config.add_skill(
            "claude-code",
            "existing-global-2",
            "https://test.com",
            "def456",
        );

        let home = dirs::home_dir().unwrap();
        for skill in ["existing-global-1", "existing-global-2"] {
            let folder = home.join(".claude/skills").join(skill);
            if folder.exists() {
                fs::remove_dir_all(&folder).unwrap();
            }
            fs::create_dir_all(&folder).unwrap();
            fs::write(folder.join("SKILL.md"), "# Skill").unwrap();
        }

        let removed = reconcile_global_config(&mut config);

        for skill in ["existing-global-1", "existing-global-2"] {
            let folder = home.join(".claude/skills").join(skill);
            if folder.exists() {
                fs::remove_dir_all(&folder).unwrap();
            }
        }

        assert!(removed.is_empty());
        assert!(config.is_skill_installed_for_tool("claude-code", "existing-global-1"));
        assert!(config.is_skill_installed_for_tool("claude-code", "existing-global-2"));
    }

    #[test]
    fn test_reconcile_global_multiple_tools() {
        let mut config = GlobalConfig::new();

        config.add_skill(
            "claude-code",
            "global-multi-1",
            "https://test.com",
            "abc123",
        );
        config.add_skill("codex", "global-multi-1", "https://test.com", "abc123");
        config.add_skill(
            "claude-code",
            "global-multi-2",
            "https://test.com",
            "def456",
        );

        let home = dirs::home_dir().unwrap();

        let claude_skill1 = home.join(".claude/skills/global-multi-1");
        if claude_skill1.exists() {
            fs::remove_dir_all(&claude_skill1).unwrap();
        }
        fs::create_dir_all(&claude_skill1).unwrap();
        fs::write(claude_skill1.join("SKILL.md"), "# Skill").unwrap();

        let claude_skill2 = home.join(".claude/skills/global-multi-2");
        if claude_skill2.exists() {
            fs::remove_dir_all(&claude_skill2).unwrap();
        }
        fs::create_dir_all(&claude_skill2).unwrap();
        fs::write(claude_skill2.join("SKILL.md"), "# Skill").unwrap();

        let removed = reconcile_global_config(&mut config);

        if claude_skill1.exists() {
            fs::remove_dir_all(&claude_skill1).unwrap();
        }
        if claude_skill2.exists() {
            fs::remove_dir_all(&claude_skill2).unwrap();
        }

        assert_eq!(removed.len(), 1);
        assert_eq!(
            removed[0],
            ("codex".to_string(), "global-multi-1".to_string())
        );

        assert!(config.is_skill_installed_for_tool("claude-code", "global-multi-1"));
        assert!(!config.is_skill_installed_for_tool("codex", "global-multi-1"));
        assert!(config.is_skill_installed_for_tool("claude-code", "global-multi-2"));
    }

    #[test]
    fn test_reconcile_empty_config() {
        let mut config = GlobalConfig::new();
        config.add_skill("claude-code", "missing-empty", "https://test.com", "abc123");

        let removed = reconcile_global_config(&mut config);

        assert_eq!(removed.len(), 1);
        assert!(config.installed_skills.is_empty());
    }
}
