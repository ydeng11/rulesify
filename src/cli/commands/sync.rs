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
            // Check if content has changed
            if rules_are_equivalent(&existing, &converted_rule) {
                Ok(SyncResult::NoChange(rule_id))
            } else {
                // Update existing rule
                if !dry_run {
                    store.save_rule(&converted_rule)?;
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
