use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

use crate::utils::config::{load_config_from_path, save_global_config, get_config_dir};

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Edit configuration file
    Edit,
    /// Set the storage directory for rules
    SetStorage { path: PathBuf },
    /// Set the default editor
    SetEditor { editor: String },
    /// Add a default tool
    AddTool { tool: String },
    /// Remove a default tool
    RemoveTool { tool: String },
}

pub fn run(action: ConfigAction, config_path: Option<PathBuf>) -> Result<()> {
    match action {
        ConfigAction::Show => show_config(config_path),
        ConfigAction::Edit => edit_config(config_path),
        ConfigAction::SetStorage { path } => set_storage_path(path, config_path),
        ConfigAction::SetEditor { editor } => set_editor(editor, config_path),
        ConfigAction::AddTool { tool } => add_default_tool(tool, config_path),
        ConfigAction::RemoveTool { tool } => remove_default_tool(tool, config_path),
    }
}

fn show_config(config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let config_dir = get_config_dir()?;

    println!("📋 Rulesify Configuration");
    println!("─────────────────────────");
    println!("📁 Config Directory: {}", config_dir.display());
    println!("📦 Rules Directory: {}", config.rules_directory.display());
    println!("✏️  Editor: {}", config.editor.as_deref().unwrap_or("(not set)"));
    println!("🔧 Default Tools: {}", config.default_tools.join(", "));

    Ok(())
}

fn edit_config(config_path: Option<PathBuf>) -> Result<()> {
    let config = load_config_from_path(config_path)?;
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join("config.yaml");

    // Ensure config file exists
    if !config_file.exists() {
        save_global_config(&config)?;
        println!("✅ Created default config file");
    }

    if let Some(editor) = &config.editor {
        println!("🖊️  Opening config in editor: {}", editor);

        let status = std::process::Command::new(editor)
            .arg(&config_file)
            .status()
            .with_context(|| format!("Failed to launch editor: {}", editor))?;

        if !status.success() {
            eprintln!("⚠️  Editor exited with error status");
        }
    } else {
        println!("📁 Config file: {}", config_file.display());
        println!("💡 Set editor with: rulesify config set-editor <editor>");
    }

    Ok(())
}

fn set_storage_path(path: PathBuf, config_path: Option<PathBuf>) -> Result<()> {
    let mut config = load_config_from_path(config_path)?;

    // Expand ~ to home directory
    let expanded_path = if path.starts_with("~") {
        let home = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?;
        home.join(path.strip_prefix("~").unwrap())
    } else {
        path
    };

    config.rules_directory = expanded_path;
    save_global_config(&config)?;

    println!("✅ Storage path set to: {}", config.rules_directory.display());

    // Create the directory if it doesn't exist
    crate::utils::fs::ensure_dir_exists(&config.rules_directory)?;

    Ok(())
}

fn set_editor(editor: String, config_path: Option<PathBuf>) -> Result<()> {
    let mut config = load_config_from_path(config_path)?;
    config.editor = Some(editor.clone());
    save_global_config(&config)?;

    println!("✅ Editor set to: {}", editor);

    Ok(())
}

fn add_default_tool(tool: String, config_path: Option<PathBuf>) -> Result<()> {
    let mut config = load_config_from_path(config_path)?;

    // Validate tool name
    let valid_tools = ["cursor", "cline", "claude-code", "goose"];
    if !valid_tools.contains(&tool.as_str()) {
        anyhow::bail!("Invalid tool: {}. Valid tools: {}", tool, valid_tools.join(", "));
    }

    if !config.default_tools.contains(&tool) {
        config.default_tools.push(tool.clone());
        save_global_config(&config)?;
        println!("✅ Added {} to default tools", tool);
    } else {
        println!("ℹ️  Tool {} is already in default tools", tool);
    }

    Ok(())
}

fn remove_default_tool(tool: String, config_path: Option<PathBuf>) -> Result<()> {
    let mut config = load_config_from_path(config_path)?;

    if let Some(pos) = config.default_tools.iter().position(|t| t == &tool) {
        config.default_tools.remove(pos);
        save_global_config(&config)?;
        println!("✅ Removed {} from default tools", tool);
    } else {
        println!("ℹ️  Tool {} was not in default tools", tool);
    }

    Ok(())
}
