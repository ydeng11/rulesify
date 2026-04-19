use crate::cli::SkillCommands;
use crate::installer::{
    install_skill, print_install_summary, print_uninstall_summary, prompt_confirm, uninstall_skill,
};
use crate::models::{get_global_config_path, GlobalConfig, ProjectConfig, Registry, Scope};
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
    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    let project_config = if project_config_path.exists() {
        let content = std::fs::read_to_string(project_config_path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Some(config)
    } else {
        None
    };

    let global_skills = global_config.list_all_skills();
    let project_skills = project_config
        .as_ref()
        .map(|c| c.list_skills())
        .unwrap_or_default();

    if global_skills.is_empty() && project_skills.is_empty() {
        println!("No skills installed.");
        println!("Run `rulesify init` for project setup, or `rulesify skill add <id> --global` for global skills.");
        return Ok(());
    }

    if !global_skills.is_empty() {
        println!("Global skills:");
        for (tool, id, info) in global_skills {
            println!("  - {} [{}] (added: {})", id, tool, info.added);
            if verbose {
                println!("    Source: {}", info.source);
            }
        }
    }

    if !project_skills.is_empty() {
        println!("\nProject skills:");
        for (id, info) in project_skills {
            println!("  - {} (added: {})", id, info.added);
            if verbose {
                println!("    Source: {}", info.source);
            }
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

    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    if global_config.is_skill_installed_globally(&id) {
        let tools = global_config.get_tools_for_skill(&id);
        println!(
            "'{}' is already installed globally for: {}",
            id,
            tools.join(", ")
        );
        if !global {
            println!("Skipping project-level installation to avoid duplication.");
        }
        return Ok(());
    }

    if !global {
        if let Some(project_config) = load_project_config(project_config_path)? {
            if project_config.installed_skills.contains_key(&id) {
                println!("'{}' is already installed at project level.", id);
                return Ok(());
            }
        }
    }

    let project_config = load_project_config(project_config_path)?;
    let tools = project_config
        .as_ref()
        .map(|c| c.tools.clone())
        .unwrap_or_default();

    if tools.is_empty() {
        return Err(RulesifyError::ConfigNotFound.into());
    }

    let registry = load_registry().await?;

    let skill = registry
        .get_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    let client = GitHubClient::new();

    println!("Installing '{}'...", skill.name);

    let results = install_skill(skill, &tools, scope.clone(), &client).await?;

    print_install_summary(&results, &skill.name);

    let success_count = results.iter().filter(|r| r.success).count();
    if success_count == 0 {
        return Err(RulesifyError::SkillParse(format!(
            "Failed to install '{}' to any tool",
            skill.name
        ))
        .into());
    }

    if global {
        let mut global_config = GlobalConfig::load();
        for tool in &tools {
            if results.iter().any(|r| r.tool == *tool && r.success) {
                global_config.add_skill(tool, &id, &skill.source_url, &skill.commit_sha);
            }
        }
        global_config.save()?;
        println!(
            "Saved global config to {}",
            get_global_config_path().display()
        );
    } else {
        let mut project_config = project_config.unwrap_or(ProjectConfig::new());
        project_config.add_skill(&id, &skill.source_url, &skill.commit_sha, Scope::Project);
        std::fs::write(
            project_config_path,
            toml::to_string_pretty(&project_config)?,
        )?;
    }

    Ok(())
}

fn remove_skill(id: String, global: bool, _verbose: bool) -> Result<()> {
    let scope = if global {
        Scope::Global
    } else {
        Scope::Project
    };

    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    if global {
        let tools = global_config.get_tools_for_skill(&id);
        if tools.is_empty() {
            println!("'{}' is not installed globally.", id);
            return Ok(());
        }

        let message = format!(
            "Delete skill folders for '{}' from {} global tools?",
            id,
            tools.len()
        );

        if !prompt_confirm(&message) {
            println!("Cancelled.");
            return Ok(());
        }

        let results = uninstall_skill(&id, &tools, scope);

        print_uninstall_summary(&results, &id);

        let mut global_config = GlobalConfig::load();
        for tool in &tools {
            global_config.remove_skill(tool, &id);
        }
        global_config.save()?;
    } else {
        let project_config = load_project_config(project_config_path)?
            .ok_or_else(|| RulesifyError::ConfigNotFound)?;

        if !project_config.installed_skills.contains_key(&id) {
            println!("'{}' is not installed at project level.", id);
            return Ok(());
        }

        let message = format!(
            "Delete skill folders for '{}' (used by {} tools)?",
            id,
            project_config.tools.len()
        );

        if !prompt_confirm(&message) {
            println!("Cancelled.");
            return Ok(());
        }

        let results = uninstall_skill(&id, &project_config.tools, scope);

        print_uninstall_summary(&results, &id);

        let mut project_config = project_config;
        project_config.remove_skill(&id);
        std::fs::write(
            project_config_path,
            toml::to_string_pretty(&project_config)?,
        )?;
    }

    Ok(())
}

async fn update_registry(verbose: bool) -> Result<()> {
    println!("Updating registry cache...");

    let registry = fetch_registry().await?;
    let cache = RegistryCache::new(Path::new("."));
    cache.save(&registry)?;

    println!("Registry updated ({} skills)", registry.skills.len());

    if verbose {
        println!("Updated date: {}", registry.updated);
    }

    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");
    let project_config = load_project_config(project_config_path)?;

    let mut global_updated: Vec<(String, String, crate::models::Skill)> = vec![];
    let mut project_updated: Vec<(String, crate::models::Skill)> = vec![];

    for (tool, id, info) in global_config.list_all_skills() {
        if let Some(skill) = registry.get_skill(&id) {
            if skill.commit_sha != info.commit_sha {
                global_updated.push((tool, id, skill.clone()));
            }
        }
    }

    if let Some(config) = &project_config {
        for (id, info) in config.installed_skills.iter() {
            if let Some(skill) = registry.get_skill(id) {
                if skill.commit_sha != info.commit_sha {
                    project_updated.push((id.clone(), skill.clone()));
                }
            }
        }
    }

    if global_updated.is_empty() && project_updated.is_empty() {
        println!("No installed skills need updates.");
        return Ok(());
    }

    println!(
        "\n{} global skills, {} project skills have updates:",
        global_updated.len(),
        project_updated.len()
    );

    for (tool, id, skill) in &global_updated {
        let old_sha = &global_config
            .get_skill_for_tool(tool, id)
            .unwrap()
            .commit_sha;
        println!(
            "  - {} [{}] (global: {} → {})",
            id, tool, old_sha, skill.commit_sha
        );
    }

    for (id, skill) in &project_updated {
        let old_sha = &project_config
            .as_ref()
            .unwrap()
            .installed_skills
            .get(id)
            .unwrap()
            .commit_sha;
        println!("  - {} (project: {} → {})", id, old_sha, skill.commit_sha);
    }

    let message = format!(
        "Update {} skills?",
        global_updated.len() + project_updated.len()
    );

    if !prompt_confirm(&message) {
        println!("Cancelled.");
        return Ok(());
    }

    let client = GitHubClient::new();

    for (tool, _id, skill) in &global_updated {
        println!("\nUpdating '{}' [{}] (global)...", skill.name, tool);
        let results = install_skill(skill, &[tool.clone()], Scope::Global, &client).await?;
        print_install_summary(&results, &skill.name);
    }

    if !project_updated.is_empty() && project_config.is_some() {
        let tools = project_config.as_ref().unwrap().tools.clone();
        for (_id, skill) in &project_updated {
            println!("\nUpdating '{}' (project)...", skill.name);
            let results = install_skill(skill, &tools, Scope::Project, &client).await?;
            print_install_summary(&results, &skill.name);
        }
    }

    let mut global_config = GlobalConfig::load();
    for (tool, id, skill) in &global_updated {
        global_config.update_skill_sha(tool, id, &skill.commit_sha);
    }
    global_config.save()?;

    if let Some(mut config) = project_config {
        for (id, skill) in &project_updated {
            config.update_skill_sha(id, &skill.commit_sha);
        }
        std::fs::write(project_config_path, toml::to_string_pretty(&config)?)?;
    }

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

fn load_project_config(path: &Path) -> Result<Option<ProjectConfig>> {
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(path)?;
    let config: ProjectConfig = toml::from_str(&content)?;
    Ok(Some(config))
}
