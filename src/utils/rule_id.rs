use anyhow::Result;
use regex::Regex;
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

/// Embeds a rule ID as an HTML comment at the top of content.
///
/// This creates a standardized comment that can be parsed back to retrieve
/// the original rule ID, ensuring proper tracking across import/sync operations.
pub fn embed_rule_id_in_content(content: &str, rule_id: &str) -> String {
    let comment = format!("<!-- rulesify-id: {} -->", rule_id);

    // If content already has a rulesify-id comment, replace it
    if content.contains("<!-- rulesify-id:") {
        let re = Regex::new(r"<!-- rulesify-id: [^>]+ -->").unwrap();
        re.replace(content, &comment).to_string()
    } else {
        // Add the comment at the top
        format!("{}\n{}", comment, content)
    }
}

/// Extracts a rule ID from embedded HTML comment in content.
///
/// Looks for the pattern `<!-- rulesify-id: {rule_id} -->` and returns the rule ID.
/// Returns None if no embedded rule ID is found.
pub fn extract_embedded_rule_id(content: &str) -> Option<String> {
    let re = Regex::new(r"<!-- rulesify-id: ([^>]+) -->").unwrap();

    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|id| !id.is_empty())
}

/// Determines the rule ID using a fallback hierarchy.
///
/// Priority order:
/// 1. Embedded rule ID in content (highest priority)
/// 2. Filename-based rule ID
/// 3. Rule name from content (sanitized)
/// 4. Default fallback ID
pub fn determine_rule_id_with_fallback(
    content: &str,
    file_path: Option<&Path>,
    rule_name: Option<&str>,
) -> Result<String> {
    // First priority: embedded rule ID
    if let Some(embedded_id) = extract_embedded_rule_id(content) {
        if is_valid_rule_id(&embedded_id) {
            return Ok(embedded_id);
        }
    }

    // Second priority: filename-based rule ID
    if let Some(path) = file_path {
        if let Ok(filename_id) = extract_rule_id_from_filename(path) {
            return Ok(filename_id);
        }
    }

    // Third priority: rule name from content
    if let Some(name) = rule_name {
        if let Ok(name_id) = sanitize_rule_id(name) {
            return Ok(name_id);
        }
    }

    // Last resort: default fallback
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(format!("imported-rule-{}", timestamp))
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

    #[test]
    fn test_embed_rule_id_in_content() {
        let content = "# My Rule\n\nSome content here";
        let result = embed_rule_id_in_content(content, "my-rule");
        assert_eq!(
            result,
            "<!-- rulesify-id: my-rule -->\n# My Rule\n\nSome content here"
        );

        // Test replacing existing embedded ID
        let content_with_id = "<!-- rulesify-id: old-rule -->\n# My Rule\n\nSome content";
        let result = embed_rule_id_in_content(content_with_id, "new-rule");
        assert_eq!(
            result,
            "<!-- rulesify-id: new-rule -->\n# My Rule\n\nSome content"
        );
    }

    #[test]
    fn test_extract_embedded_rule_id() {
        let content = "<!-- rulesify-id: my-rule -->\n# My Rule\n\nSome content";
        assert_eq!(
            extract_embedded_rule_id(content),
            Some("my-rule".to_string())
        );

        let content_no_id = "# My Rule\n\nSome content";
        assert_eq!(extract_embedded_rule_id(content_no_id), None);

        let content_empty_id = "<!-- rulesify-id:  -->\n# My Rule";
        assert_eq!(extract_embedded_rule_id(content_empty_id), None);
    }

    #[test]
    fn test_determine_rule_id_with_fallback() {
        // Test embedded rule ID (highest priority)
        let content_with_embedded = "<!-- rulesify-id: embedded-rule -->\n# Some Rule";
        let path = PathBuf::from("filename-rule.md");
        let result = determine_rule_id_with_fallback(
            content_with_embedded,
            Some(&path),
            Some("Content Rule Name"),
        )
        .unwrap();
        assert_eq!(result, "embedded-rule");

        // Test filename fallback
        let content_no_embedded = "# Some Rule\n\nContent";
        let result = determine_rule_id_with_fallback(
            content_no_embedded,
            Some(&path),
            Some("Content Rule Name"),
        )
        .unwrap();
        assert_eq!(result, "filename-rule");

        // Test rule name fallback
        let result =
            determine_rule_id_with_fallback(content_no_embedded, None, Some("Content Rule Name"))
                .unwrap();
        assert_eq!(result, "content-rule-name");

        // Test default fallback
        let result = determine_rule_id_with_fallback(content_no_embedded, None, None).unwrap();
        assert!(result.starts_with("imported-rule-"));
    }
}
