use crate::installer::{generate_instructions, generate_uninstall_instructions_batch};
use crate::models::{ProjectConfig, Registry, Scope};
use crate::registry::{fetch_registry, load_builtin, RegistryCache};
use crate::scanner::scan_project;
use crate::tui::{SelectionResult, SkillSelector, ToolPicker};
use crate::utils::Result;
use std::collections::HashSet;
use std::path::Path;

pub async fn run(verbose: bool) -> Result<()> {
    let project_path = Path::new(".");
    let config_path = Path::new(".rulesify.toml");

    if verbose {
        println!("Scanning project...");
    }
    let context = scan_project(project_path)?;

    if verbose {
        println!("Languages: {:?}", context.languages);
        println!("Frameworks: {:?}", context.frameworks);
        println!("Existing tools: {:?}", context.existing_tools);
    }

    let existing_config = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        if verbose {
            println!(
                "Loaded existing config with {} installed skills",
                config.installed_skills.len()
            );
        }
        Some(config)
    } else {
        None
    };

    let existing_tools = existing_config
        .as_ref()
        .map(|c| c.tools.clone())
        .unwrap_or_default();

    println!("Select AI tools you use:");
    let tools = ToolPicker::run_with_selected(existing_tools)?;

    if tools.is_empty() {
        println!("No tools selected. Exiting.");
        return Ok(());
    }

    let registry = load_registry().await?;

    if registry.skills.is_empty() {
        println!("No skills available in registry.");
        return Ok(());
    }

    let installed_ids: HashSet<String> = existing_config
        .as_ref()
        .map(|c| c.installed_skills.keys().cloned().collect())
        .unwrap_or_default();

    let skills_to_show: Vec<_> = registry
        .skills
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    println!("\nSelect skills ([i] = already installed, [x] = newly selected):");
    let result: SelectionResult = SkillSelector::new(skills_to_show, installed_ids).run()?;

    if result.selected.is_empty() {
        println!("No skills selected. Exiting.");
        return Ok(());
    }

    if !result.added.is_empty() {
        let install_instructions = generate_instructions(&result.added, &tools);
        println!("\n## Skills to Install:\n{}", install_instructions);
    }

    if !result.removed.is_empty() {
        let uninstall_instructions =
            generate_uninstall_instructions_batch(&result.removed, &tools, Scope::Project);
        println!("\n## Skills to Remove:\n{}", uninstall_instructions);
    }

    if result.added.is_empty() && result.removed.is_empty() {
        println!("\nNo changes to installed skills.");
    }

    let mut config = existing_config.unwrap_or(ProjectConfig::new());
    config.tools = tools;

    for id in &result.removed {
        config.remove_skill(id);
    }

    for (id, skill) in &result.added {
        config.add_skill(id, &skill.source_url, Scope::Project);
    }

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;
    println!("\nSaved configuration to .rulesify.toml");

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
