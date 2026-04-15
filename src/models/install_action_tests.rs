#[cfg(test)]
mod tests {
    use crate::models::InstallAction;

    #[test]
    fn test_copy_action_with_folder() {
        let action = InstallAction::Copy {
            folder: "debugging".to_string(),
        };
        assert!(action.is_simple());
    }

    #[test]
    fn test_command_action() {
        let action = InstallAction::Command {
            value: "git clone https://example.com".to_string(),
        };
        assert!(!action.is_simple());
    }
}
