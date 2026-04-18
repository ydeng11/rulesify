use crate::installer::generate_instructions;
use crate::models::{ProjectConfig, Registry, Scope};
use crate::registry::{fetch_registry, load_builtin, RegistryCache};
use crate::scanner::scan_project;
use crate::tui::{SkillSelector, ToolPicker};
use crate::utils::Result;
use std::path::Path;

pub async fn run(verbose: bool) -> Result<()> {
    let project_path = Path::new(".");

    if verbose {
        println!("Scanning project...");
    }
    let context = scan_project(project_path)?;

    if verbose {
        println!("Languages: {:?}", context.languages);
        println!("Frameworks: {:?}", context.frameworks);
        println!("Existing tools: {:?}", context.existing_tools);
    }

    println!("Select AI tools you use:");
    let tools = ToolPicker::run()?;

    if tools.is_empty() {
        println!("No tools selected. Exiting.");
        return Ok(());
    }

    let registry = load_registry().await?;

    if registry.skills.is_empty() {
        println!("No skills available in registry.");
        return Ok(());
    }

    let skills_to_show: Vec<_> = registry
        .skills
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    println!("\nSelect skills to install:");
    let selected = SkillSelector::new(skills_to_show).run()?;

    if selected.is_empty() {
        println!("No skills selected. Exiting.");
        return Ok(());
    }

    let instructions = generate_instructions(&selected, &tools);
    println!("\n{}", instructions);

    let mut config = ProjectConfig::new();
    config.tools = tools;
    for (id, skill) in &selected {
        config.add_skill(id, &skill.source_url, Scope::Project);
    }

    std::fs::write(".rulesify.toml", toml::to_string_pretty(&config)?)?;
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
