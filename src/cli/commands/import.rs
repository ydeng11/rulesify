use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::converters::{
    claude_code::ClaudeCodeConverter, cline::ClineConverter, cursor::CursorConverter,
    goose::GooseConverter, RuleConverter,
};
use crate::store::{file_store::FileStore, RuleStore};
use crate::utils::config::load_config_from_path;
use crate::utils::rule_id::determine_rule_id_with_fallback;

pub fn run(
    tool: String,
    file: PathBuf,
    rule_id: Option<String>,
    config_path: Option<PathBuf>,
) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Validate tool
    let converter = get_converter(&tool)?;

    // Check if file exists
    if !file.exists() {
        anyhow::bail!("File not found: {}", file.display());
    }

    // Read the file content
    let content = fs::read_to_string(&file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    // Convert from tool format to URF
    let mut rule = converter
        .convert_from_tool_format(&content)
        .with_context(|| format!("Failed to convert {} format to URF", tool))?;

    // Determine final rule ID: CLI override > embedded ID > filename > content-based fallback
    let final_rule_id = if let Some(custom_id) = rule_id {
        // User provided explicit override
        custom_id
    } else {
        // Use the new fallback hierarchy to determine rule ID
        let determined_id =
            determine_rule_id_with_fallback(&content, Some(&file), Some(&rule.metadata.name))
                .with_context(|| {
                    format!("Cannot determine rule ID from file: {}", file.display())
                })?;

        // Check if determined ID differs from content-based ID
        if determined_id != rule.id {
            println!(
                "ℹ️  Note: Using determined rule ID '{}' (content suggests '{}')",
                determined_id, rule.id
            );
            println!("   Use --rule-id to override this behavior");
        }

        determined_id
    };

    // Override the rule ID with our determined value
    rule.id = final_rule_id;

    // Check if rule already exists
    if store.load_rule(&rule.id)?.is_some() {
        print!("⚠️  Rule '{}' already exists. Overwrite? [y/N]: ", rule.id);
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
            println!("Import cancelled");
            return Ok(());
        }
    }

    // Save the rule
    store
        .save_rule(&rule)
        .with_context(|| format!("Failed to save rule '{}'", rule.id))?;

    println!("✅ Successfully imported rule: {}", rule.id);
    println!("📄 Name: {}", rule.metadata.name);
    if let Some(description) = &rule.metadata.description {
        println!("📝 Description: {}", description);
    }
    println!("📁 URF file: {}", store.get_rule_path(&rule.id).display());

    // Offer to open in editor
    if let Some(editor) = &config.editor {
        print!("🖊️  Open rule in editor? [y/N]: ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
            let rule_path = store.get_rule_path(&rule.id);
            let status = std::process::Command::new(editor)
                .arg(&rule_path)
                .status()
                .with_context(|| format!("Failed to launch editor: {}", editor))?;

            if !status.success() {
                eprintln!("⚠️  Editor exited with error status");
            }
        }
    }

    Ok(())
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
