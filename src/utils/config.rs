use crate::models::config::GlobalConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn get_config_dir() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Unable to determine home directory"))?;
    
    Ok(home_dir.join(".rulesify"))
}

pub fn load_global_config() -> Result<GlobalConfig> {
    let config_dir = get_config_dir()?;
    let config_file = config_dir.join("config.yaml");
    
    if !config_file.exists() {
        // Return default config
        return Ok(GlobalConfig {
            rules_directory: config_dir.join("rules"),
            editor: std::env::var("EDITOR").ok(),
            default_tools: vec!["cursor".to_string(), "cline".to_string()],
        });
    }
    
    let content = fs::read_to_string(&config_file)
        .with_context(|| format!("Failed to read config file: {}", config_file.display()))?;
    
    let config: GlobalConfig = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse config file")?;
    
    Ok(config)
}

pub fn save_global_config(config: &GlobalConfig) -> Result<()> {
    let config_dir = get_config_dir()?;
    crate::utils::fs::ensure_dir_exists(&config_dir)?;
    
    let config_file = config_dir.join("config.yaml");
    let content = serde_yaml::to_string(config)
        .with_context(|| "Failed to serialize config")?;
    
    fs::write(&config_file, content)
        .with_context(|| format!("Failed to write config file: {}", config_file.display()))?;
    
    Ok(())
} 