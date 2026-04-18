use crate::cli::SkillCommands;
use crate::installer::{
    install_skill, print_install_summary, print_uninstall_summary, prompt_confirm, uninstall_skill,
};
use crate::models::{ProjectConfig, Registry, Scope};
use crate::registry::{fetch_registry, load_builtin, GitHubClient, RegistryCache};
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

    let client = GitHubClient::new();

    println!("Installing '{}'...", skill.name);

    let results = install_skill(skill, &config.tools, scope.clone(), &client).await?;

    print_install_summary(&results, &skill.name);

    let success_count = results.iter().filter(|r| r.success).count();
    if success_count > 0 {
        config.add_skill(&id, &skill.source_url, &skill.commit_sha, scope);
        std::fs::write(config_path, toml::to_string_pretty(&config)?)?;
    }

    if success_count == 0 {
        return Err(RulesifyError::SkillParse(format!(
            "Failed to install '{}' to any tool",
            skill.name
        ))
        .into());
    }

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
    let config: ProjectConfig = toml::from_str(&content)?;

    if !config.installed_skills.contains_key(&id) {
        return Err(RulesifyError::SkillNotFound(id.clone()).into());
    }

    let message = format!(
        "Delete skill folders for '{}' (used by {} tools)?",
        id,
        config.tools.len()
    );

    if !prompt_confirm(&message) {
        println!("Cancelled.");
        return Ok(());
    }

    let results = uninstall_skill(&id, &config.tools, scope.clone());

    print_uninstall_summary(&results, &id);

    let mut config: ProjectConfig = toml::from_str(&content)?;
    config.remove_skill(&id);
    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

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

    let config_path = Path::new(".rulesify.toml");

    if !config_path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(config_path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;

    let mut updated_skills: Vec<(String, crate::models::Skill)> = vec![];

    for (id, info) in config.installed_skills.iter() {
        if let Some(skill) = registry.get_skill(id) {
            if skill.commit_sha != info.commit_sha {
                updated_skills.push((id.clone(), skill.clone()));
            }
        }
    }

    if updated_skills.is_empty() {
        println!("No installed skills need updates.");
        return Ok(());
    }

    println!("\n{} skills have updates:", updated_skills.len());

    for (id, skill) in &updated_skills {
        println!(
            "  - {} (old: {}, new: {})",
            id,
            config.installed_skills.get(id).unwrap().commit_sha,
            skill.commit_sha
        );
    }

    let message = format!("Update {} skills?", updated_skills.len());

    if !prompt_confirm(&message) {
        println!("Cancelled.");
        return Ok(());
    }

    let client = GitHubClient::new();

    for (id, skill) in &updated_skills {
        println!("\nUpdating '{}'...", skill.name);
        let scope = config.installed_skills.get(id).unwrap().scope.clone();
        let results = install_skill(skill, &config.tools, scope.clone(), &client).await?;
        print_install_summary(&results, &skill.name);
        config.update_skill_sha(id, &skill.commit_sha);
    }

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;

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
