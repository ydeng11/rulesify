use crate::cli::SkillCommands;
use crate::installer::{generate_install_instructions, generate_uninstall_instructions};
use crate::models::{ProjectConfig, Registry, Scope};
use crate::registry::{fetch_registry, load_builtin, RegistryCache};
use crate::utils::{Result, RulesifyError};
use std::path::Path;

pub async fn run(command: SkillCommands, verbose: bool) -> Result<()> {
    match command {
        SkillCommands::List => list_skills(verbose),
        SkillCommands::Add { id, global } => add_skill(id, global, verbose).await,
        SkillCommands::Remove { id, global } => remove_skill(id, global, verbose),
        SkillCommands::Update => update_registry(verbose).await,
    }
}

fn list_skills(verbose: bool) -> Result<()> {
    let config_path = Path::new(".rulesify.toml");

    if !config_path.exists() {
        println!("No skills installed. Run `rulesify init` first.");
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    let config: ProjectConfig = toml::from_str(&content)?;

    println!("Installed skills:");
    for (id, info) in config.list_skills() {
        let scope_label = match info.scope {
            Scope::Project => "project",
            Scope::Global => "global",
        };
        println!("  - {} (added: {}, scope: {})", id, info.added, scope_label);
        if verbose {
            println!("    Source: {}", info.source);
        }
    }

    Ok(())
}

async fn add_skill(id: String, global: bool, _verbose: bool) -> Result<()> {
    let scope = if global {
        Scope::Global
    } else {
        Scope::Project
    };
    let config_path = Path::new(".rulesify.toml");

    if !config_path.exists() {
        return Err(RulesifyError::ConfigNotFound.into());
    }

    let registry = load_registry().await?;

    let skill = registry
        .get_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    let content = std::fs::read_to_string(config_path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;

    let instructions =
        generate_install_instructions(&skill.name, &skill.source_url, &config.tools, scope.clone());

    config.add_skill(&id, &skill.source_url, scope);

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

    println!("Added skill: {}", skill.name);
    println!("Source: {}", skill.source_url);
    println!("\n{}", instructions);

    Ok(())
}

fn remove_skill(id: String, global: bool, _verbose: bool) -> Result<()> {
    let scope = if global {
        Scope::Global
    } else {
        Scope::Project
    };
    let config_path = Path::new(".rulesify.toml");

    if !config_path.exists() {
        return Err(RulesifyError::ConfigNotFound.into());
    }

    let content = std::fs::read_to_string(config_path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;

    let removed = config
        .remove_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    let instructions = generate_uninstall_instructions(&id, &config.tools, scope.clone());

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

    println!("Removed skill: {}", id);
    println!("Added on: {}", removed.added);
    println!("\n{}", instructions);

    Ok(())
}

async fn update_registry(verbose: bool) -> Result<()> {
    println!("Updating registry cache...");

    let registry = fetch_registry().await?;
    let cache = RegistryCache::new();
    cache.save(&registry)?;

    println!("Registry updated ({} skills)", registry.skills.len());

    if verbose {
        println!("Updated date: {}", registry.updated);
    }

    Ok(())
}

async fn load_registry() -> Result<Registry> {
    let cache = RegistryCache::new();

    if let Ok(registry) = fetch_registry().await {
        cache.save(&registry)?;
        return Ok(registry);
    }

    if let Some(registry) = cache.load()? {
        return Ok(registry);
    }

    load_builtin()
}
