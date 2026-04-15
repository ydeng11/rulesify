#[cfg(test)]
mod tests {
    use crate::models::InstallAction;

    #[test]
    fn test_copy_action() {
        let action = InstallAction::Copy {
            path: "tdd/SKILL.md".to_string(),
        };
        assert!(action.is_simple());
        assert_eq!(
            action.install_command("https://github.com/test/skills"),
            Some("rulesify skill fetch https://github.com/test/skills/tdd/SKILL.md".to_string())
        );
    }

    #[test]
    fn test_command_action() {
        let action = InstallAction::Command {
            value: "git clone https://example.com ~/.agents/skills/foo".to_string(),
        };
        assert!(!action.is_simple());
        assert_eq!(
            action.install_command("https://github.com/test"),
            Some("git clone https://example.com ~/.agents/skills/foo".to_string())
        );
    }
}
