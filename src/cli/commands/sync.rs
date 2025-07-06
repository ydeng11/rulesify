use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::converters::{
    claude_code::ClaudeCodeConverter, cline::ClineConverter, cursor::CursorConverter,
    goose::GooseConverter, RuleConverter,
};
use crate::models::rule::UniversalRule;
use crate::store::{file_store::FileStore, RuleStore};
use crate::utils::config::load_config_from_path;

pub fn run(
    dry_run: bool,
    rule: Option<String>,
    tool: Option<String>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory.clone());

    if dry_run {
        println!("🔍 Running in dry-run mode (no changes will be made)");
    }

    // Determine which tools to sync from
    let source_tools = if let Some(tool_name) = tool {
        vec![tool_name]
    } else {
        config.default_tools.clone()
    };

    println!("🔄 Syncing deployed rules back to URF format");

    let project_root = std::env::current_dir().context("Failed to get current directory")?;

    let mut synced_count = 0;
    let mut created_count = 0;

    for tool_name in &source_tools {
        println!("\n📋 Checking {} rules", tool_name);

        let converter = get_converter(tool_name)?;
        let deployment_path = converter.get_deployment_path(&project_root);

        if !deployment_path.exists() {
            println!(
                "  ⏭️  No {} rules found at {}",
                tool_name,
                deployment_path.display()
            );
            continue;
        }

        // Find deployed rule files
        let deployed_files = find_deployed_files(&deployment_path, &converter)?;

        for file_path in deployed_files {
            if let Some(rule_name) = &rule {
                // Only sync specific rule if requested
                if !file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s == rule_name)
                    .unwrap_or(false)
                {
                    continue;
                }
            }

            match sync_rule_from_file(&store, converter.as_ref(), &file_path, dry_run) {
                Ok(SyncResult::Updated(rule_id)) => {
                    println!("  ✅ Updated URF: {}", rule_id);
                    synced_count += 1;
                }
                Ok(SyncResult::Created(rule_id)) => {
                    println!("  ✨ Created URF: {}", rule_id);
                    created_count += 1;
                }
                Ok(SyncResult::NoChange(rule_id)) => {
                    println!("  ⏭️  No changes: {}", rule_id);
                }
                Err(e) => {
                    println!("  ❌ Error syncing {}: {}", file_path.display(), e);
                }
            }
        }
    }

    if dry_run {
        println!("\n🔍 Dry run complete - no changes made");
    } else {
        println!(
            "\n🎉 Sync complete: {} updated, {} created",
            synced_count, created_count
        );
    }

    Ok(())
}

enum SyncResult {
    Updated(String),
    Created(String),
    NoChange(String),
}

fn get_converter(tool_name: &str) -> Result<Box<dyn RuleConverter>> {
    match tool_name.to_lowercase().as_str() {
        "cursor" => Ok(Box::new(CursorConverter::new())),
        "cline" => Ok(Box::new(ClineConverter::new())),
        "claude-code" | "claude_code" => Ok(Box::new(ClaudeCodeConverter::new())),
        "goose" => Ok(Box::new(GooseConverter::new())),
        _ => anyhow::bail!(
            "Unsupported tool: {}. Supported tools: cursor, cline, claude-code, goose",
            tool_name
        ),
    }
}

fn find_deployed_files(
    deployment_path: &Path,
    converter: &Box<dyn RuleConverter>,
) -> Result<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();

    if deployment_path.is_file() {
        files.push(deployment_path.to_path_buf());
    } else if deployment_path.is_dir() {
        let entries = fs::read_dir(deployment_path)
            .with_context(|| format!("Failed to read directory: {}", deployment_path.display()))?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == converter.get_file_extension() {
                        files.push(path);
                    }
                }
            }
        }
    }

    Ok(files)
}

fn sync_rule_from_file(
    store: &FileStore,
    converter: &dyn RuleConverter,
    file_path: &Path,
    dry_run: bool,
) -> Result<SyncResult> {
    // Read the deployed file
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

    // Extract rule ID from filename
    let rule_id = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid filename: {}", file_path.display()))?
        .to_string();

    // Check if URF already exists
    let existing_rule = store.load_rule(&rule_id)?;

    // Convert from tool format to URF
    let mut converted_rule = converter.convert_from_tool_format(&content)?;

    // Override the converted rule's ID with the filename-based ID
    // This ensures we sync back to the correct URF file
    converted_rule.id = rule_id.clone();

    match existing_rule {
        Some(existing) => {
            // Preserve original metadata that isn't available in tool format
            // Only update fields that can be determined from the tool format
            converted_rule.metadata.tags = existing.metadata.tags.clone();
            converted_rule.metadata.priority = existing.metadata.priority;
            converted_rule.version = existing.version.clone();
            converted_rule.tool_overrides = existing.tool_overrides.clone();

            // Check if content has changed
            if rules_are_equivalent(&existing, &converted_rule) {
                Ok(SyncResult::NoChange(rule_id))
            } else {
                // Update existing rule with selective merging to preserve comments
                if !dry_run {
                    update_urf_file_selectively(store, &rule_id, &existing, &converted_rule)?;
                }
                Ok(SyncResult::Updated(rule_id))
            }
        }
        None => {
            // Create new URF - but warn user they should use import instead
            println!("  ⚠️  Warning: No existing URF found for '{}'. Consider using 'rulesify import' instead.", rule_id);
            if !dry_run {
                store.save_rule(&converted_rule)?;
            }
            Ok(SyncResult::Created(rule_id))
        }
    }
}

fn rules_are_equivalent(rule1: &UniversalRule, rule2: &UniversalRule) -> bool {
    // Compare key fields (ignoring timestamps and metadata that might differ)
    rule1.metadata.name == rule2.metadata.name
        && rule1.metadata.description == rule2.metadata.description
        && rule1.content == rule2.content
        && rule1.references == rule2.references
        && rule1.conditions == rule2.conditions
}

/// Updates a URF file selectively, preserving comments and formatting,
/// while only updating the fields that have actually changed
fn update_urf_file_selectively(
    store: &FileStore,
    rule_id: &str,
    existing_rule: &UniversalRule,
    updated_rule: &UniversalRule,
) -> Result<()> {
    let rule_path = store.get_rule_path(rule_id);

    // Read the original URF file content to preserve formatting and comments
    let original_content = fs::read_to_string(&rule_path)
        .with_context(|| format!("Failed to read original URF file: {}", rule_path.display()))?;

    let mut updated_content = original_content.clone();

    // Update metadata fields if they changed
    if existing_rule.metadata.name != updated_rule.metadata.name {
        updated_content = update_yaml_field(
            &updated_content,
            "metadata.name",
            &format!("\"{}\"", updated_rule.metadata.name),
        )?;
    }

    if existing_rule.metadata.description != updated_rule.metadata.description {
        let description_value = match &updated_rule.metadata.description {
            Some(desc) => {
                if desc.contains('\n') {
                    format!(
                        "|\n{}",
                        desc.lines()
                            .map(|line| format!("    {}", line))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    format!("\"{}\"", desc)
                }
            }
            None => "null".to_string(),
        };
        updated_content =
            update_yaml_field(&updated_content, "metadata.description", &description_value)?;
    }

    if existing_rule.metadata.tags != updated_rule.metadata.tags {
        let tags_value = if updated_rule.metadata.tags.is_empty() {
            "[]".to_string()
        } else {
            format!(
                "[{}]",
                updated_rule
                    .metadata
                    .tags
                    .iter()
                    .map(|tag| format!("\"{}\"", tag))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        updated_content = update_yaml_field(&updated_content, "metadata.tags", &tags_value)?;
    }

    if existing_rule.metadata.priority != updated_rule.metadata.priority {
        updated_content = update_yaml_field(
            &updated_content,
            "metadata.priority",
            &updated_rule.metadata.priority.to_string(),
        )?;
    }

    if existing_rule.metadata.auto_apply != updated_rule.metadata.auto_apply {
        updated_content = update_yaml_field(
            &updated_content,
            "metadata.auto_apply",
            &updated_rule.metadata.auto_apply.to_string(),
        )?;
    }

    // For content, references, and conditions changes, fall back to complete file replacement
    // but preserve the original file structure by doing a smart merge
    if existing_rule.content != updated_rule.content
        || existing_rule.references != updated_rule.references
        || existing_rule.conditions != updated_rule.conditions
    {
        // If structural changes are detected, fall back to saving the complete rule
        // while preserving the original file's comments and structure where possible
        return fallback_to_complete_update(store, rule_id, updated_rule, &original_content);
    }

    // Write the updated content back to the file
    fs::write(&rule_path, updated_content)
        .with_context(|| format!("Failed to write updated URF file: {}", rule_path.display()))?;

    Ok(())
}

/// Updates a specific YAML field in the content while preserving formatting
fn update_yaml_field(content: &str, field_path: &str, new_value: &str) -> Result<String> {
    use regex::Regex;

    let field_parts: Vec<&str> = field_path.split('.').collect();

    if field_parts.len() == 2 && field_parts[0] == "metadata" {
        let field_name = field_parts[1];
        let pattern = format!(r"(\s*{}\s*:\s*)([^\n]+)", regex::escape(field_name));
        let regex = Regex::new(&pattern)
            .with_context(|| format!("Failed to create regex for field {}", field_name))?;

        if regex.is_match(content) {
            let result = regex.replace(content, format!("$1{}", new_value));
            Ok(result.to_string())
        } else {
            // Field doesn't exist, we'll let the normal save handle it
            Ok(content.to_string())
        }
    } else {
        // For non-metadata fields, fall back to normal replacement
        Ok(content.to_string())
    }
}

/// Fallback to complete file update when structural changes are detected
/// This preserves the original file structure while updating changed content
fn fallback_to_complete_update(
    store: &FileStore,
    rule_id: &str,
    updated_rule: &UniversalRule,
    original_content: &str,
) -> Result<()> {
    let rule_path = store.get_rule_path(rule_id);

    // Parse the original content line by line to find section boundaries
    let original_lines: Vec<&str> = original_content.lines().collect();
    let mut result_lines = Vec::new();
    let mut i = 0;

    // Copy lines until we find the content section
    while i < original_lines.len() {
        let line = original_lines[i];
        if line.trim() == "content:" {
            // Found the content section - add the content header
            result_lines.push(line.to_string());
            i += 1;

            // Skip all existing content lines until we find the next top-level section
            while i < original_lines.len() {
                let line = original_lines[i];
                // Check if this is the start of a new top-level section
                if line.starts_with(char::is_alphabetic)
                    && line.contains(':')
                    && !line.starts_with(' ')
                    && !line.starts_with('\t')
                {
                    // This is the next section, don't increment i so we process it
                    break;
                }
                i += 1;
            }

            // Add the new content
            let updated_content_yaml = serde_yaml::to_string(&updated_rule.content)
                .with_context(|| "Failed to serialize updated content")?;

            // Add each line of the new content
            for line in updated_content_yaml.lines() {
                result_lines.push(line.to_string());
            }

            // Continue processing from the next section
            continue;
        }

        result_lines.push(line.to_string());
        i += 1;
    }

    // Join the lines back together
    let result_content = result_lines.join("\n");

    // Write the updated content
    fs::write(&rule_path, result_content)
        .with_context(|| format!("Failed to write updated URF file: {}", rule_path.display()))?;

    Ok(())
}
