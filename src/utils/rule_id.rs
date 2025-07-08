use anyhow::Result;
use std::path::Path;

/// Sanitizes a rule name or filename to create a valid rule ID.
///
/// This function ensures that rule IDs are:
/// - Lowercase with no spaces
/// - Use hyphens instead of underscores or spaces
/// - Contain only alphanumeric characters and hyphens
/// - At least 2 characters long
/// - At most 50 characters long
///
/// Used consistently across rule creation, import, and sync operations.
pub fn sanitize_rule_id(input: &str) -> Result<String> {
    let sanitized = input
        .to_lowercase()
        .replace(' ', "-")
        .replace('_', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    // Remove multiple consecutive hyphens
    let cleaned = sanitized
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if cleaned.is_empty() {
        anyhow::bail!(
            "Input '{}' results in empty rule ID after sanitization",
            input
        );
    }

    if cleaned.len() < 2 {
        anyhow::bail!("Rule ID '{}' must be at least 2 characters long", cleaned);
    }

    if cleaned.len() > 50 {
        anyhow::bail!("Rule ID '{}' must be 50 characters or less", cleaned);
    }

    Ok(cleaned)
}

/// Extracts and sanitizes a rule ID from a file path.
///
/// Takes the filename (without extension) and applies the same sanitization
/// rules as sanitize_rule_id(). Used by import and sync commands.
pub fn extract_rule_id_from_filename(file_path: &Path) -> Result<String> {
    let stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", file_path.display()))?;

    sanitize_rule_id(stem)
}

/// Validates if a string is already a valid rule ID.
///
/// Returns true if the input follows rule ID conventions:
/// - Lowercase
/// - Only alphanumeric and hyphens
/// - 2-50 characters
/// - No consecutive hyphens
pub fn is_valid_rule_id(input: &str) -> bool {
    if input.len() < 2 || input.len() > 50 {
        return false;
    }

    if input.contains("--") {
        return false;
    }

    input
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !input.starts_with('-')
        && !input.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_sanitize_rule_id() {
        assert_eq!(sanitize_rule_id("Simple Rule").unwrap(), "simple-rule");
        assert_eq!(
            sanitize_rule_id("TypeScript_Style").unwrap(),
            "typescript-style"
        );
        assert_eq!(
            sanitize_rule_id("REACT-patterns").unwrap(),
            "react-patterns"
        );
        assert_eq!(sanitize_rule_id("test123").unwrap(), "test123");
        assert_eq!(
            sanitize_rule_id("Rule With Spaces").unwrap(),
            "rule-with-spaces"
        );
        assert_eq!(
            sanitize_rule_id("Rule__With___Underscores").unwrap(),
            "rule-with-underscores"
        );
        assert_eq!(
            sanitize_rule_id("Rule!@#$%^&*()Special").unwrap(),
            "rulespecial"
        );
    }

    #[test]
    fn test_sanitize_rule_id_errors() {
        assert!(sanitize_rule_id("").is_err());
        assert!(sanitize_rule_id("!@#$").is_err());
        assert!(sanitize_rule_id("a").is_err());
        assert!(sanitize_rule_id(&"x".repeat(51)).is_err());
    }

    #[test]
    fn test_extract_rule_id_from_filename() {
        let path = PathBuf::from("my-rule.mdc");
        assert_eq!(extract_rule_id_from_filename(&path).unwrap(), "my-rule");

        let path = PathBuf::from("React_Components.md");
        assert_eq!(
            extract_rule_id_from_filename(&path).unwrap(),
            "react-components"
        );

        let path = PathBuf::from("CODING STANDARDS.goosehints");
        assert_eq!(
            extract_rule_id_from_filename(&path).unwrap(),
            "coding-standards"
        );
    }

    #[test]
    fn test_is_valid_rule_id() {
        assert!(is_valid_rule_id("simple-rule"));
        assert!(is_valid_rule_id("typescript-style"));
        assert!(is_valid_rule_id("rule123"));
        assert!(is_valid_rule_id("ab"));

        assert!(!is_valid_rule_id("Simple-Rule")); // uppercase
        assert!(!is_valid_rule_id("rule with spaces")); // spaces
        assert!(!is_valid_rule_id("rule--double")); // double hyphens
        assert!(!is_valid_rule_id("-rule")); // starts with hyphen
        assert!(!is_valid_rule_id("rule-")); // ends with hyphen
        assert!(!is_valid_rule_id("a")); // too short
        assert!(!is_valid_rule_id(&"x".repeat(51))); // too long
    }
}
