use anyhow::{Context, Result};
use clap::Subcommand;
use log::{debug, error, info};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::store::{file_store::FileStore, RuleStore};
use crate::templates::builtin::create_skeleton_for_rule;
use crate::utils::config::load_config_from_path;

#[derive(Subcommand, Debug)]
pub enum RuleAction {
    /// Create a new rule from skeleton
    New { name: String },
    /// Edit an existing rule
    Edit { name: String },
    /// List all rules
    List {
        #[arg(short, long)]
        regex: Option<String>,
    },
    /// Show rule details
    Show { name: String },
    /// Delete a rule
    Delete { name: String },
}

pub fn run(action: RuleAction, config_path: Option<PathBuf>) -> Result<()> {
    match action {
        RuleAction::New { name } => create_new_rule(&name, config_path),
        RuleAction::Edit { name } => edit_rule(&name, config_path),
        RuleAction::List { regex } => list_rules(regex, config_path),
        RuleAction::Show { name } => show_rule(&name, config_path),
        RuleAction::Delete { name } => delete_rule(&name, config_path),
    }
}

fn create_new_rule(name: &str, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Check if rule already exists
    if store.load_rule(name)?.is_some() {
        anyhow::bail!("Rule '{}' already exists", name);
    }

    // Create skeleton YAML content
    let skeleton_content = create_skeleton_for_rule(name)?;

    // Write the skeleton directly to preserve comments
    let rule_path = store.get_rule_path(name);
    fs::write(&rule_path, skeleton_content)
        .with_context(|| format!("Failed to write rule file: {}", rule_path.display()))?;

    println!("✅ Created new rule: {}", name);
    println!("📁 File location: {}", store.get_rule_path(name).display());

    // Open in editor if available
    if let Some(editor) = &config.editor {
        let rule_path = store.get_rule_path(name);
        println!("🖊️  Opening in editor: {}", editor);

        let status = Command::new(editor)
            .arg(&rule_path)
            .status()
            .with_context(|| format!("Failed to launch editor: {}", editor))?;

        if !status.success() {
            eprintln!("⚠️  Editor exited with error status");
        }
    } else {
        println!("💡 Set EDITOR environment variable to auto-open rules for editing");
    }

    Ok(())
}

fn edit_rule(name: &str, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Check if rule exists
    if store.load_rule(name)?.is_none() {
        anyhow::bail!("Rule '{}' not found", name);
    }

    let rule_path = store.get_rule_path(name);

    if let Some(editor) = &config.editor {
        println!("🖊️  Opening '{}' in editor: {}", name, editor);

        let status = Command::new(editor)
            .arg(&rule_path)
            .status()
            .with_context(|| format!("Failed to launch editor: {}", editor))?;

        if !status.success() {
            eprintln!("⚠️  Editor exited with error status");
        }
    } else {
        println!("📁 Rule file: {}", rule_path.display());
        println!("💡 Set EDITOR environment variable to auto-open rules for editing");
    }

    Ok(())
}

fn list_rules(regex_pattern: Option<String>, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    let rule_ids = store.list_rules()?;

    if rule_ids.is_empty() {
        println!("No rules found. Create one with: rulesify rule new <name>");
        return Ok(());
    }

    let filtered_rules = if let Some(pattern) = regex_pattern {
        let regex =
            Regex::new(&pattern).with_context(|| format!("Invalid regex pattern: {}", pattern))?;

        rule_ids
            .into_iter()
            .filter(|id| regex.is_match(id))
            .collect::<Vec<_>>()
    } else {
        rule_ids
    };

    if filtered_rules.is_empty() {
        println!("No rules match the given pattern");
        return Ok(());
    }

    println!("📋 Rules ({})", filtered_rules.len());
    println!("{}", "─".repeat(40));

    for rule_id in &filtered_rules {
        // Load rule to get metadata
        match store.load_rule(rule_id)? {
            Some(rule) => {
                println!("• {} - {}", rule_id, rule.metadata.name);
                if let Some(description) = &rule.metadata.description {
                    let short_desc = if description.len() > 60 {
                        format!("{}...", &description[..57])
                    } else {
                        description.clone()
                    };
                    println!("  {}", short_desc);
                }
            }
            None => {
                println!("• {} - [Error loading rule]", rule_id);
            }
        }
    }

    Ok(())
}

fn show_rule(name: &str, config_path: Option<PathBuf>) -> Result<()> {
    debug!("Showing rule: {}", name);

    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    let rule = store.load_rule(name)?.ok_or_else(|| {
        error!("Rule '{}' not found", name);
        anyhow::anyhow!("Rule '{}' not found", name)
    })?;

    println!("📄 Rule: {}", rule.metadata.name);
    println!("🆔 ID: {}", rule.id);
    println!("📦 Version: {}", rule.version);

    if let Some(description) = &rule.metadata.description {
        println!("📝 Description: {}", description);
    }

    if !rule.metadata.tags.is_empty() {
        println!("🏷️  Tags: {}", rule.metadata.tags.join(", "));
    }

    println!("⚡ Priority: {}", rule.metadata.priority);

    // Check for cursor-specific auto_apply setting
    let cursor_auto_apply = rule
        .tool_overrides
        .get("cursor")
        .and_then(|cursor_overrides| cursor_overrides.get("auto_apply"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    println!("🔄 Auto-apply (Cursor): {}", cursor_auto_apply);

    if !rule.content.is_empty() {
        println!("\n📖 Content:");
        for (i, section) in rule.content.iter().enumerate() {
            println!("  {}. {} ({:?})", i + 1, section.title, section.format);
            // Show first few lines of content
            let lines: Vec<&str> = section.value.lines().take(3).collect();
            for line in lines {
                if line.trim().is_empty() {
                    continue;
                }
                println!("     {}", line);
            }
            if section.value.lines().count() > 3 {
                println!("     ...");
            }
        }
    }

    if !rule.references.is_empty() {
        println!("\n📎 References:");
        for reference in &rule.references {
            println!("  @{}", reference.path);
        }
    }

    if !rule.conditions.is_empty() {
        println!("\n🎯 Conditions: {} pattern(s)", rule.conditions.len());
    }

    println!("\n📁 File: {}", store.get_rule_path(name).display());

    info!("Successfully showed rule: {}", name);
    Ok(())
}

fn delete_rule(name: &str, config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let store = FileStore::new(config.rules_directory);

    // Check if rule exists
    if store.load_rule(name)?.is_none() {
        anyhow::bail!("Rule '{}' not found", name);
    }

    // Confirm deletion
    print!(
        "⚠️  Are you sure you want to delete rule '{}'? [y/N]: ",
        name
    );
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().to_lowercase() != "y" && input.trim().to_lowercase() != "yes" {
        println!("Deletion cancelled");
        return Ok(());
    }

    // Delete the rule
    store
        .delete_rule(name)
        .with_context(|| format!("Failed to delete rule '{}'", name))?;

    println!("✅ Deleted rule: {}", name);

    Ok(())
}
