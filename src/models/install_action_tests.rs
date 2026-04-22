#[cfg(test)]
mod tests {
    use crate::models::InstallAction;

    #[test]
    fn test_copy_action_with_folder() {
        let action = InstallAction::Copy {
            folder: "debugging".to_string(),
        };
        assert!(action.is_simple());
        assert!(!action.is_mega_skill_copy());
    }

    #[test]
    fn test_command_action() {
        let action = InstallAction::Command {
            value: "git clone https://example.com".to_string(),
        };
        assert!(!action.is_simple());
        assert!(!action.is_mega_skill_copy());
    }

    #[test]
    fn test_mega_skill_copy_action() {
        let action = InstallAction::mega_skill_copy("skills", "superpowers");
        assert!(action.is_mega_skill_copy());
        assert!(!action.is_simple());
        assert!(!action.is_npx());
    }

    #[test]
    fn test_mega_skill_copy_serialization() {
        let action = InstallAction::mega_skill_copy("source/skills", "impeccable");
        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("mega-skill-copy"));
        assert!(serialized.contains("source/skills"));
        assert!(serialized.contains("impeccable"));

        let deserialized: InstallAction = serde_json::from_str(&serialized).unwrap();
        assert!(deserialized.is_mega_skill_copy());
    }

    #[test]
    fn test_npx_action() {
        let action = InstallAction::Npx {
            package: "get-shit-done-cc".to_string(),
            args: vec!["@latest".to_string()],
            uninstall_flag: None,
        };
        assert!(action.is_npx());
        assert!(!action.is_mega_skill_copy());
        assert!(!action.is_simple());
    }
}
