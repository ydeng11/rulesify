use anyhow::{Context, Result};
use log::{debug, error, info};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::converters::{
    claude_code::ClaudeCodeConverter, cline::ClineConverter, cursor::CursorConverter,
    goose::GooseConverter, RuleConverter,
};
use crate::models::rule::{RuleCondition, RuleContent, RuleMetadata, UniversalRule};
use crate::store::{file_store::FileStore, RuleStore};
use crate::utils::config::load_config_from_path;
use crate::utils::rule_id::sanitize_rule_id;

pub fn run(
    tool: Option<String>,
    rule: Option<String>,
    all: bool,
    config_path: Option<PathBuf>,
) -> Result<()> {
    debug!(
        "Deploy command started with tool: {:?}, rule: {:?}, all: {}",
        tool, rule, all
    );

    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Determine which tools to deploy to
    let target_tools = if let Some(tool_name) = tool {
        vec![tool_name]
    } else {
        config.default_tools
    };

    // Validate all tools before proceeding
    for tool_name in &target_tools {
        debug!("Validating tool: {}", tool_name);
        get_converter(tool_name)?; // This will fail early if tool is invalid
    }

    // Determine which rules to deploy
    let rule_names = if all {
        store.list_rules()?
    } else if let Some(rule_spec) = rule {
        // Parse comma-separated rule names
        let names: Vec<String> = rule_spec
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if names.is_empty() {
            anyhow::bail!("No valid rule names provided");
        }

        // Validate all rules exist
        for rule_name in &names {
            if store.load_rule(rule_name)?.is_none() {
                anyhow::bail!("Rule '{}' not found", rule_name);
            }
        }

        names
    } else {
        anyhow::bail!("Must specify either --rule <name[,name,...]> or --all");
    };

    if rule_names.is_empty() {
        println!("No rules found to deploy");
        return Ok(());
    }

    println!(
        "🚀 Deploying {} rule(s) to {} tool(s)",
        rule_names.len(),
        target_tools.len()
    );

    let mut deployment_errors = Vec::new();

    for tool_name in &target_tools {
        println!("\n📋 Deploying to {}", tool_name);

        let converter = get_converter(tool_name)?; // This should already be validated above
        let project_root = std::env::current_dir().context("Failed to get current directory")?;
        let deployment_path = converter.get_deployment_path(&project_root);

        // Check if we have multiple rules - if so, we need to merge them
        if rule_names.len() > 1 {
            // Load all rules that need to be merged
            let mut rules_to_merge = Vec::new();
            for rule_name in &rule_names {
                let rule = store
                    .load_rule(rule_name)?
                    .ok_or_else(|| anyhow::anyhow!("Rule '{}' not found", rule_name))?;
                rules_to_merge.push(rule);
            }

            // Show merge preview
            show_merge_preview(&rules_to_merge);

            // Prompt for merged rule ID
            let merged_rule_id = prompt_for_merged_rule_id(&rule_names)?;

            // Check if merged rule ID conflicts with existing rules
            if store.load_rule(&merged_rule_id)?.is_some() {
                print!(
                    "⚠️  Rule '{}' already exists. Overwrite? [y/N]: ",
                    merged_rule_id
                );
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
                    println!("Deployment cancelled");
                    continue; // Skip this tool and continue with next
                }
            }

            // Create merged rule
            let merged_rule = merge_rules(rules_to_merge, merged_rule_id.clone())?;

            // Deploy the merged rule
            match deploy_merged_rule(&merged_rule, converter.as_ref(), &deployment_path) {
                Ok(output_path) => {
                    println!(
                        "  ✅ Merged {} rules → {}",
                        rule_names.len(),
                        output_path.display()
                    );
                    info!(
                        "Successfully deployed merged rule '{}' to {}",
                        merged_rule_id,
                        output_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("  ❌ Merge deployment failed: {}", e);
                    error!("Failed to deploy merged rule '{}': {}", merged_rule_id, e);
                    deployment_errors.push(format!("Merged rule '{}': {}", merged_rule_id, e));
                }
            }
        } else {
            // Single rule deployment (existing logic)
            let rule_name = &rule_names[0];
            match deploy_rule(&store, converter.as_ref(), rule_name, &deployment_path) {
                Ok(output_path) => {
                    println!("  ✅ {} → {}", rule_name, output_path.display());
                    info!(
                        "Successfully deployed rule '{}' to {}",
                        rule_name,
                        output_path.display()
                    );
                }
                Err(e) => {
                    eprintln!("  ❌ {} failed: {}", rule_name, e);
                    error!("Failed to deploy rule '{}': {}", rule_name, e);
                    deployment_errors.push(format!("Rule '{}': {}", rule_name, e));
                }
            }
        }
    }

    if !deployment_errors.is_empty() {
        anyhow::bail!(
            "Deployment failed for {} rule(s): {}",
            deployment_errors.len(),
            deployment_errors.join("; ")
        );
    }

    println!("\n🎉 Deployment complete!");
    Ok(())
}

fn get_converter(tool_name: &str) -> Result<Box<dyn RuleConverter>> {
    debug!("Getting converter for tool: {}", tool_name);
    match tool_name.to_lowercase().as_str() {
        "cursor" => Ok(Box::new(CursorConverter::new())),
        "cline" => Ok(Box::new(ClineConverter::new())),
        "claude-code" | "claude_code" => Ok(Box::new(ClaudeCodeConverter::new())),
        "goose" => Ok(Box::new(GooseConverter::new())),
        _ => {
            error!("Unsupported tool: {}", tool_name);
            anyhow::bail!(
                "Unsupported tool: {}. Supported tools: cursor, cline, claude-code, goose",
                tool_name
            )
        }
    }
}

fn merge_rules(rules: Vec<UniversalRule>, new_id: String) -> Result<UniversalRule> {
    if rules.is_empty() {
        anyhow::bail!("Cannot merge empty list of rules");
    }

    if rules.len() == 1 {
        let mut rule = rules.into_iter().next().unwrap();
        rule.id = new_id;
        return Ok(rule);
    }

    // Sort rules by priority (highest first: 10 → 1)
    let mut sorted_rules = rules;
    sorted_rules.sort_by(|a, b| b.metadata.priority.cmp(&a.metadata.priority));

    let highest_priority_rule = &sorted_rules[0];

    // Merge metadata
    let name = highest_priority_rule.metadata.name.clone();

    // Append descriptions with separator
    let description = {
        let descriptions: Vec<String> = sorted_rules
            .iter()
            .filter_map(|rule| rule.metadata.description.as_ref())
            .cloned()
            .collect();

        if descriptions.is_empty() {
            None
        } else {
            Some(descriptions.join("\n\n---\n\n"))
        }
    };

    // Combine and deduplicate tags (preserve order from highest priority)
    let tags = {
        let mut seen_tags = HashSet::new();
        let mut combined_tags = Vec::new();

        for rule in &sorted_rules {
            for tag in &rule.metadata.tags {
                if seen_tags.insert(tag.clone()) {
                    combined_tags.push(tag.clone());
                }
            }
        }

        combined_tags
    };

    let priority = highest_priority_rule.metadata.priority;

    // Use highest priority rule's tool_overrides
    let tool_overrides = highest_priority_rule.tool_overrides.clone();

    // Combine content sections in priority order
    let mut content = Vec::new();
    for rule in &sorted_rules {
        // Add a header comment to identify the source rule
        content.push(RuleContent {
            title: format!("From: {}", rule.metadata.name),
            format: crate::models::rule::ContentFormat::Markdown,
            value: format!(
                "*The following sections are from rule: {}*",
                rule.metadata.name
            ),
        });

        // Add all content sections from this rule
        for section in &rule.content {
            content.push(section.clone());
        }
    }

    // Merge references and deduplicate
    let references = {
        let mut seen_refs = HashSet::new();
        let mut combined_refs = Vec::new();

        for rule in &sorted_rules {
            for reference in &rule.references {
                if seen_refs.insert(reference.path.clone()) {
                    combined_refs.push(reference.clone());
                }
            }
        }

        combined_refs
    };

    // Merge conditions and deduplicate
    let conditions = {
        let mut seen_conditions = HashSet::new();
        let mut combined_conditions = Vec::new();

        for rule in &sorted_rules {
            for condition in &rule.conditions {
                let condition_key = match condition {
                    RuleCondition::FilePattern { value } => format!("file_pattern:{}", value),
                    RuleCondition::Regex { value } => format!("regex:{}", value),
                };

                if seen_conditions.insert(condition_key) {
                    combined_conditions.push(condition.clone());
                }
            }
        }

        combined_conditions
    };

    // Use the version from the highest priority rule
    let version = highest_priority_rule.version.clone();

    Ok(UniversalRule {
        id: new_id,
        version,
        metadata: RuleMetadata {
            name,
            description,
            tags,
            priority,
        },
        content,
        references,
        conditions,
        tool_overrides,
    })
}

fn deploy_rule(
    store: &FileStore,
    converter: &dyn RuleConverter,
    rule_name: &str,
    deployment_path: &Path,
) -> Result<std::path::PathBuf> {
    // Load the rule
    let rule = store
        .load_rule(rule_name)?
        .ok_or_else(|| anyhow::anyhow!("Rule '{}' not found", rule_name))?;

    // Convert to tool format
    let tool_content = converter
        .convert_to_tool_format(&rule)
        .with_context(|| format!("Failed to convert rule '{}' to tool format", rule_name))?;

    // Determine output file path
    let output_path = if deployment_path.is_dir() || deployment_path.extension().is_none() {
        // This is a directory path - append the filename
        deployment_path.join(format!("{}.{}", rule_name, converter.get_file_extension()))
    } else {
        // This is a file path - use as-is or modify for special cases
        match converter.get_file_extension() {
            "md" if deployment_path.file_name().unwrap_or_default() == "CLAUDE.md" => {
                deployment_path.to_path_buf()
            }
            "goosehints" => deployment_path.with_file_name(".goosehints"),
            _ => deployment_path.to_path_buf(),
        }
    };

    // Ensure the parent directory of the output file exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write the converted content
    fs::write(&output_path, tool_content)
        .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

    Ok(output_path)
}

fn deploy_merged_rule(
    merged_rule: &UniversalRule,
    converter: &dyn RuleConverter,
    deployment_path: &Path,
) -> Result<std::path::PathBuf> {
    // Convert to tool format
    let tool_content = converter
        .convert_to_tool_format(merged_rule)
        .with_context(|| {
            format!(
                "Failed to convert merged rule '{}' to tool format",
                merged_rule.id
            )
        })?;

    // Determine output file path
    let output_path = if deployment_path.is_dir() || deployment_path.extension().is_none() {
        // This is a directory path - append the filename
        deployment_path.join(format!(
            "{}.{}",
            merged_rule.id,
            converter.get_file_extension()
        ))
    } else {
        // This is a file path - use as-is or modify for special cases
        match converter.get_file_extension() {
            "md" if deployment_path.file_name().unwrap_or_default() == "CLAUDE.md" => {
                deployment_path.to_path_buf()
            }
            "goosehints" => deployment_path.with_file_name(".goosehints"),
            _ => deployment_path.to_path_buf(),
        }
    };

    // Ensure the parent directory of the output file exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    // Write the converted content
    fs::write(&output_path, tool_content)
        .with_context(|| format!("Failed to write file: {}", output_path.display()))?;

    Ok(output_path)
}

fn prompt_for_merged_rule_id(rule_names: &[String]) -> Result<String> {
    println!("\n📦 Multiple rules detected for merging:");
    for (i, rule_name) in rule_names.iter().enumerate() {
        println!("  {}. {}", i + 1, rule_name);
    }

    print!("\n🔗 Enter ID for the merged rule: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        anyhow::bail!("Merged rule ID cannot be empty");
    }

    // Sanitize the rule ID
    let sanitized_id = sanitize_rule_id(trimmed_input)
        .with_context(|| format!("Invalid rule ID: '{}'", trimmed_input))?;

    if sanitized_id != trimmed_input {
        println!("ℹ️  Rule ID sanitized to: '{}'", sanitized_id);
    }

    Ok(sanitized_id)
}

fn show_merge_preview(rules: &[UniversalRule]) {
    println!("\n📋 Merge Preview:");
    println!("Rules will be combined in priority order (highest first):");

    let mut sorted_rules = rules.to_vec();
    sorted_rules.sort_by(|a, b| b.metadata.priority.cmp(&a.metadata.priority));

    for (i, rule) in sorted_rules.iter().enumerate() {
        println!(
            "  {}. {} (priority: {})",
            i + 1,
            rule.metadata.name,
            rule.metadata.priority
        );
        if let Some(desc) = &rule.metadata.description {
            let short_desc = if desc.len() > 60 {
                format!("{}...", &desc[..57])
            } else {
                desc.clone()
            };
            println!("     {}", short_desc);
        }
    }

    // Show combined tags
    let mut seen_tags = HashSet::new();
    let mut combined_tags = Vec::new();
    for rule in &sorted_rules {
        for tag in &rule.metadata.tags {
            if seen_tags.insert(tag.clone()) {
                combined_tags.push(tag.clone());
            }
        }
    }

    if !combined_tags.is_empty() {
        println!("📋 Combined tags: {}", combined_tags.join(", "));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::rule::{ContentFormat, RuleContent, RuleMetadata, UniversalRule};
    use std::collections::HashMap;

    fn create_test_rule(
        id: &str,
        name: &str,
        description: &str,
        priority: u8,
        tags: Vec<&str>,
    ) -> UniversalRule {
        UniversalRule {
            id: id.to_string(),
            version: "1.0.0".to_string(),
            metadata: RuleMetadata {
                name: name.to_string(),
                description: Some(description.to_string()),
                tags: tags.iter().map(|s| s.to_string()).collect(),
                priority,
            },
            content: vec![RuleContent {
                title: format!("{} Section", name),
                format: ContentFormat::Markdown,
                value: format!("Content for {}", name),
            }],
            references: vec![],
            conditions: vec![],
            tool_overrides: {
                let mut overrides = HashMap::new();
                let mut cursor_config = serde_json::Map::new();
                cursor_config.insert(
                    "apply_mode".to_string(),
                    serde_json::Value::String("intelligent".to_string()),
                );
                overrides.insert(
                    "cursor".to_string(),
                    serde_json::Value::Object(cursor_config),
                );
                overrides
            },
        }
    }

    #[test]
    fn test_merge_rules_priority_ordering() {
        let high_rule = create_test_rule(
            "high",
            "High Priority",
            "High priority rule",
            8,
            vec!["high", "important"],
        );
        let low_rule = create_test_rule(
            "low",
            "Low Priority",
            "Low priority rule",
            3,
            vec!["low", "optional"],
        );
        let medium_rule = create_test_rule(
            "medium",
            "Medium Priority",
            "Medium priority rule",
            6,
            vec!["medium", "standard"],
        );

        let rules = vec![low_rule.clone(), high_rule.clone(), medium_rule.clone()]; // Intentionally out of order
        let merged = merge_rules(rules, "merged-test".to_string()).unwrap();

        // Should use highest priority rule's metadata
        assert_eq!(merged.metadata.name, "High Priority");
        assert_eq!(merged.metadata.priority, 8);

        // Should use highest priority rule's tool_overrides
        assert_eq!(merged.tool_overrides, high_rule.tool_overrides);
    }

    #[test]
    fn test_merge_rules_description_concatenation() {
        let rule1 = create_test_rule("rule1", "Rule One", "First description", 5, vec![]);
        let rule2 = create_test_rule("rule2", "Rule Two", "Second description", 3, vec![]);
        let rule3 = create_test_rule("rule3", "Rule Three", "Third description", 7, vec![]);

        let rules = vec![rule1, rule2, rule3];
        let merged = merge_rules(rules, "merged-test".to_string()).unwrap();

        let expected_description =
            "Third description\n\n---\n\nFirst description\n\n---\n\nSecond description";
        assert_eq!(
            merged.metadata.description,
            Some(expected_description.to_string())
        );
    }

    #[test]
    fn test_merge_rules_tag_deduplication() {
        let rule1 = create_test_rule(
            "rule1",
            "Rule One",
            "Description",
            8,
            vec!["typescript", "style", "linting"],
        );
        let rule2 = create_test_rule(
            "rule2",
            "Rule Two",
            "Description",
            6,
            vec!["typescript", "testing", "quality"],
        ); // typescript is duplicate
        let rule3 = create_test_rule(
            "rule3",
            "Rule Three",
            "Description",
            4,
            vec!["style", "documentation"],
        ); // style is duplicate

        let rules = vec![rule1, rule2, rule3];
        let merged = merge_rules(rules, "merged-test".to_string()).unwrap();

        // Tags should be deduplicated but preserve order from highest priority rule first
        let expected_tags = vec![
            "typescript",
            "style",
            "linting",
            "testing",
            "quality",
            "documentation",
        ];
        assert_eq!(merged.metadata.tags, expected_tags);
    }

    #[test]
    fn test_merge_rules_content_combination() {
        let rule1 = create_test_rule("rule1", "Rule One", "Description", 8, vec![]);
        let rule2 = create_test_rule("rule2", "Rule Two", "Description", 6, vec![]);

        let rules = vec![rule1, rule2];
        let merged = merge_rules(rules, "merged-test".to_string()).unwrap();

        // Should have source headers plus original content
        assert_eq!(merged.content.len(), 4);

        // First should be source header for highest priority rule
        assert_eq!(merged.content[0].title, "From: Rule One");
        assert!(merged.content[0].value.contains("Rule One"));

        // Second should be content from highest priority rule
        assert_eq!(merged.content[1].title, "Rule One Section");

        // Third should be source header for second highest priority rule
        assert_eq!(merged.content[2].title, "From: Rule Two");

        // Fourth should be content from second highest priority rule
        assert_eq!(merged.content[3].title, "Rule Two Section");
    }

    #[test]
    fn test_merge_single_rule() {
        let rule = create_test_rule("single", "Single Rule", "Description", 5, vec!["tag"]);
        let rules = vec![rule.clone()];

        let merged = merge_rules(rules, "new-id".to_string()).unwrap();

        // Should just change the ID
        assert_eq!(merged.id, "new-id");
        assert_eq!(merged.metadata.name, "Single Rule");
        assert_eq!(merged.metadata.description, Some("Description".to_string()));
        assert_eq!(merged.metadata.tags, vec!["tag"]);
    }

    #[test]
    fn test_merge_empty_rules() {
        let result = merge_rules(vec![], "test".to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot merge empty list"));
    }
}
