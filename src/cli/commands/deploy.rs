use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::converters::{
    RuleConverter,
    cursor::CursorConverter,
    cline::ClineConverter,
    claude_code::ClaudeCodeConverter,
    goose::GooseConverter,
};
use crate::store::{RuleStore, file_store::FileStore};
use crate::utils::config::load_config_from_path;

pub fn run(tool: Option<String>, rule: Option<String>, all: bool, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Determine which tools to deploy to
    let target_tools = if let Some(tool_name) = tool {
        vec![tool_name]
    } else {
        config.default_tools
    };

    // Determine which rules to deploy
    let rule_names = if all {
        store.list_rules()?
    } else if let Some(rule_name) = rule {
        vec![rule_name]
    } else {
        anyhow::bail!("Must specify either --rule <name> or --all");
    };

    if rule_names.is_empty() {
        println!("No rules found to deploy");
        return Ok(());
    }

    println!("🚀 Deploying {} rule(s) to {} tool(s)", rule_names.len(), target_tools.len());

    for tool_name in &target_tools {
        println!("\n📋 Deploying to {}", tool_name);

        let converter = get_converter(tool_name)?;
        let project_root = std::env::current_dir()
            .context("Failed to get current directory")?;
                let deployment_path = converter.get_deployment_path(&project_root);

        for rule_name in &rule_names {
            match deploy_rule(&store, converter.as_ref(), rule_name, &deployment_path) {
                Ok(output_path) => {
                    println!("  ✅ {} → {}", rule_name, output_path.display());
                }
                Err(e) => {
                    eprintln!("  ❌ {} failed: {}", rule_name, e);
                }
            }
        }
    }

    println!("\n🎉 Deployment complete!");
    Ok(())
}

fn get_converter(tool_name: &str) -> Result<Box<dyn RuleConverter>> {
    match tool_name.to_lowercase().as_str() {
        "cursor" => Ok(Box::new(CursorConverter::new())),
        "cline" => Ok(Box::new(ClineConverter::new())),
        "claude-code" | "claude_code" => Ok(Box::new(ClaudeCodeConverter::new())),
        "goose" => Ok(Box::new(GooseConverter::new())),
        _ => anyhow::bail!("Unsupported tool: {}. Supported tools: cursor, cline, claude-code, goose", tool_name),
    }
}

fn deploy_rule(
    store: &FileStore,
    converter: &dyn RuleConverter,
    rule_name: &str,
    deployment_path: &Path,
) -> Result<std::path::PathBuf> {
    // Load the rule
    let rule = store.load_rule(rule_name)?
        .ok_or_else(|| anyhow::anyhow!("Rule '{}' not found", rule_name))?;

    // Convert to tool format
    let tool_content = converter.convert_to_tool_format(&rule)
        .with_context(|| format!("Failed to convert rule '{}' to tool format", rule_name))?;

    // Determine output file path
    let output_path = if deployment_path.is_dir() || deployment_path.extension().is_none() {
        // This is a directory path - append the filename
        deployment_path.join(format!("{}.{}", rule_name, converter.get_file_extension()))
    } else {
        // This is a file path - use as-is or modify for special cases
        match converter.get_file_extension() {
            "md" if deployment_path.file_name().unwrap_or_default() == "CLAUDE.md" => deployment_path.to_path_buf(),
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
