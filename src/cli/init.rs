use crate::installer::{
    install_skill, print_install_summary, print_uninstall_summary, uninstall_skill,
};
use crate::models::{GlobalConfig, ProjectConfig, Registry, Scope};
use crate::registry::{fetch_registry, load_builtin, GitHubClient, RegistryCache};
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

    let global_config = GlobalConfig::load();

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

    let project_installed_ids: HashSet<String> = existing_config
        .as_ref()
        .map(|c| c.installed_skills.keys().cloned().collect())
        .unwrap_or_default();

    let global_installed_ids: HashSet<String> = tools
        .iter()
        .flat_map(|tool| {
            global_config
                .list_skills_for_tool(tool)
                .into_iter()
                .map(|(id, _)| id)
        })
        .collect();

    let skills_to_show: Vec<_> = registry
        .skills
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    println!("\nSelect skills ([g] = global, [i] = installed, [x] = newly selected):");
    let result: SelectionResult =
        SkillSelector::new(skills_to_show, project_installed_ids, global_installed_ids).run()?;

    if result.selected.is_empty() {
        println!("No skills selected. Exiting.");
        return Ok(());
    }

    let client = GitHubClient::new();
    let mut config = existing_config.unwrap_or(ProjectConfig::new());
    config.tools = tools.clone();

    if !result.removed.is_empty() {
        println!("\nRemoving {} skills...", result.removed.len());
        for id in &result.removed {
            let results = uninstall_skill(id, &tools, Scope::Project);
            print_uninstall_summary(&results, id);
            config.remove_skill(id);
        }
    }

    if !result.added.is_empty() {
        println!("\nInstalling {} skills...", result.added.len());
        for (id, skill) in &result.added {
            if global_config.is_skill_installed_globally(id) {
                let tools_for_skill = global_config.get_tools_for_skill(id);
                println!(
                    "'{}' is already installed globally for: {}, skipping.",
                    skill.name,
                    tools_for_skill.join(", ")
                );
                continue;
            }

            println!("Installing '{}'...", skill.name);
            let results = install_skill(skill, &tools, Scope::Project, &client).await?;
            print_install_summary(&results, &skill.name);
            config.add_skill(id, &skill.source_url, &skill.commit_sha, Scope::Project);
        }
    }

    if result.added.is_empty() && result.removed.is_empty() {
        println!("\nNo changes to installed skills.");
    }

    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;
    println!("\nSaved configuration to .rulesify.toml");

    Ok(())
}

async fn load_registry() -> Result<Registry> {
    let cache = RegistryCache::new(Path::new("."));

    if let Ok(registry) = fetch_registry().await {
        cache.save(&registry)?;
        return Ok(registry);
    }

    if let Some(registry) = cache.load()? {
        return Ok(registry);
    }

    load_builtin()
}
