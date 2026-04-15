use crate::cli::SkillCommands;
use crate::models::{ProjectConfig, Registry};
use crate::registry::{fetch_registry, load_builtin, RegistryCache};
use crate::utils::{Result, RulesifyError};
use std::path::Path;

pub async fn run(command: SkillCommands, verbose: bool) -> Result<()> {
    match command {
        SkillCommands::List => list_skills(verbose),
        SkillCommands::Add { id } => add_skill(id, verbose).await,
        SkillCommands::Remove { id } => remove_skill(id, verbose),
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
        println!("  - {} (added: {})", id, info.added);
        if verbose {
            println!("    Source: {}", info.source);
        }
    }

    Ok(())
}

async fn add_skill(id: String, _verbose: bool) -> Result<()> {
    let registry = load_registry().await?;

    let skill = registry
        .get_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    let config_path = Path::new(".rulesify.toml");
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        toml::from_str::<ProjectConfig>(&content)?
    } else {
        ProjectConfig::new()
    };

    config.add_skill(&id, &skill.source_url);

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

    println!("Added skill: {}", skill.name);
    println!("Source: {}", skill.source_url);
    println!("\nInstall instructions:");
    println!(
        "  Fetch from {} and add to your AI tool config",
        skill.source_url
    );

    Ok(())
}

fn remove_skill(id: String, _verbose: bool) -> Result<()> {
    let config_path = Path::new(".rulesify.toml");

    if !config_path.exists() {
        println!("No skills installed.");
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;

    let removed = config
        .remove_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

    println!("Removed skill: {}", id);
    println!("Added on: {}", removed.added);
    println!("\nCleanup instructions:");
    println!("  Remove skill content from your AI tool config files");

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
