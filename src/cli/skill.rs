use crate::cli::SkillCommands;
use crate::fetcher::ArchiveCache;
use crate::installer::{
    execute_npx_install, generate_install_instructions, generate_uninstall_instructions,
    install_mega_skill, install_skill, print_install_summary, print_uninstall_summary,
    resolve_pi_coverage, uninstall_skill,
};
use crate::models::{
    get_global_config_path, GlobalConfig, InstallAction, ProjectConfig, Registry, Scope,
};
use crate::registry::{fetch_registry, load_builtin, GitHubClient, RegistryCache};
use crate::utils::{check_all_dependencies, Result, RulesifyError};
use std::path::Path;

pub async fn run(command: SkillCommands, verbose: bool) -> Result<()> {
    match command {
        SkillCommands::List => list_skills(verbose),
        SkillCommands::Search { query } => search_skills(query, verbose),
        SkillCommands::Add {
            id,
            global,
            agent_mode,
        } => add_skill(id, global, agent_mode, verbose).await,
        SkillCommands::Remove {
            id,
            global,
            agent_mode,
        } => remove_skill(id, global, agent_mode, verbose),
        SkillCommands::Update { agent_mode, force } => {
            update_directory_registry(agent_mode, force, verbose).await
        }
    }
}

fn coverage_suffix(covered_tools: &[String]) -> String {
    if covered_tools.is_empty() {
        String::new()
    } else {
        format!(" [covers: {}]", covered_tools.join(", "))
    }
}

fn list_skills(verbose: bool) -> Result<()> {
    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    let project_config = load_project_config(project_config_path)?;

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
            println!(
                "  - {} [{}] (added: {}){}",
                id,
                tool,
                info.added,
                coverage_suffix(&info.covered_tools)
            );
            if verbose {
                println!("    Source: {}", info.source);
            }
        }
    }

    if !project_skills.is_empty() {
        println!("\nProject skills:");
        for (id, info) in project_skills {
            println!(
                "  - {} (added: {}){}",
                id,
                info.added,
                coverage_suffix(&info.covered_tools)
            );
            if verbose {
                println!("    Source: {}", info.source);
                if !info.covered_tools.is_empty() {
                    println!("    Covered tools: {}", info.covered_tools.join(", "));
                }
            }
        }
    }

    Ok(())
}

fn search_skills(query: Option<String>, verbose: bool) -> Result<()> {
    let registry = load_builtin()?;

    let skills: Vec<_> = if let Some(q) = query {
        registry
            .skills
            .iter()
            .filter(|(_, skill)| {
                skill.name.to_lowercase().contains(&q.to_lowercase())
                    || skill.description.to_lowercase().contains(&q.to_lowercase())
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        registry
            .skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    println!("Available skills ({} total):\n", skills.len());

    let mut mega_skills: Vec<_> = skills.iter().filter(|(_, s)| s.is_mega_skill).collect();
    mega_skills.sort_by(|a, b| b.1.stars.cmp(&a.1.stars));

    let mut regular_skills: Vec<_> = skills.iter().filter(|(_, s)| !s.is_mega_skill).collect();
    regular_skills.sort_by(|a, b| b.1.name.cmp(&a.1.name));

    if !mega_skills.is_empty() {
        println!("Mega-Skills (skill collections):");
        for (id, skill) in mega_skills {
            let score_text = skill
                .score
                .map(|s| format!("{:.0}", s))
                .unwrap_or_else(|| "-".to_string());
            println!("  [M] {} - {}", skill.name, skill.description);
            if verbose {
                println!("      ID: {}", id);
                println!("      Stars: ★{}", skill.stars);
                println!("      Score: {}", score_text);
                println!("      Source: {}", skill.source_url);
            }
        }
        println!();
    }

    println!("Regular Skills:");
    for (id, skill) in regular_skills {
        let score_text = skill
            .score
            .map(|s| format!("{:.0}", s))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "  {} - {}",
            skill.name,
            skill.description.lines().next().unwrap_or("")
        );
        if verbose {
            println!("      ID: {}", id);
            println!("      Domain: {}", skill.domain);
            println!("      Stars: ★{}", skill.stars);
            println!("      Score: {}", score_text);
            println!("      Tags: {}", skill.tags.join(", "));
        }
    }

    println!("\nTo install: rulesify skill add <id>");
    println!("For mega-skills: rulesify skill add <name> --global");

    Ok(())
}

async fn add_skill(id: String, global: bool, agent_mode: bool, _verbose: bool) -> Result<()> {
    let scope = if global {
        Scope::Global
    } else {
        Scope::Project
    };

    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    if !agent_mode && global_config.is_skill_installed_globally(&id) {
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

    if !agent_mode && !global {
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

    let (physical_tools, covered_tools) = resolve_pi_coverage(&tools);

    let registry = load_registry().await?;

    let skill = registry
        .get_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;

    if agent_mode {
        output_install_instructions(skill, &tools, scope);
        return Ok(());
    }

    let missing_deps = check_all_dependencies(&skill.dependencies);
    if !missing_deps.is_empty() {
        return Err(RulesifyError::DependencyMissing {
            dependency: missing_deps.join(", "),
            skill: skill.name.clone(),
        }
        .into());
    }

    if !covered_tools.is_empty() {
        println!(
            "Pi is covered by other agents — skipping physical install for pi, marking in registry."
        );
    }

    println!("Installing '{}'...", skill.name);

    let results = match &skill.install_action {
        Some(InstallAction::Npx {
            package,
            args,
            uninstall_flag,
        }) => execute_npx_install(
            package,
            args,
            uninstall_flag.as_deref(),
            &physical_tools,
            scope,
        )?,
        Some(InstallAction::Copy { .. }) | None => {
            let cache = ArchiveCache::new();
            let client = GitHubClient::new();
            install_skill(skill, &physical_tools, scope, &client, &cache).await?
        }
        Some(InstallAction::MegaSkillCopy {
            source_folder,
            dest_name,
        }) => {
            let cache = ArchiveCache::new();
            let client = GitHubClient::new();
            install_mega_skill(
                skill,
                source_folder,
                dest_name,
                &physical_tools,
                scope,
                &client,
                &cache,
            )
            .await?
        }
        Some(InstallAction::Command { value }) => {
            println!("Running custom install command: {}", value);
            // Still register covered tool entries
            if global {
                let mut global_config = GlobalConfig::load();
                for tool in &physical_tools {
                    global_config.add_skill(
                        tool,
                        &id,
                        &skill.source_url,
                        &skill.commit_sha,
                        covered_tools.clone(),
                    );
                }
                global_config.save()?;
            }
            return Ok(());
        }
    };

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
        for tool in &physical_tools {
            if results.iter().any(|r| r.tool == *tool && r.success) {
                global_config.add_skill(
                    tool,
                    &id,
                    &skill.source_url,
                    &skill.commit_sha,
                    covered_tools.clone(),
                );
            }
        }
        global_config.save()?;
        println!(
            "Saved global config to {}",
            get_global_config_path().display()
        );
    } else {
        let mut project_config = project_config.unwrap_or(ProjectConfig::new());
        project_config.add_skill(
            &id,
            &skill.source_url,
            &skill.commit_sha,
            Scope::Project,
            covered_tools.clone(),
        );
        std::fs::write(
            project_config_path,
            toml::to_string_pretty(&project_config)?,
        )?;
    }

    Ok(())
}

fn output_install_instructions(skill: &crate::models::Skill, tools: &[String], scope: Scope) {
    println!(
        "{}",
        generate_install_instructions(&skill.name, &skill.source_url, tools, scope)
    );

    if let Some(InstallAction::Npx {
        package,
        args,
        uninstall_flag,
    }) = &skill.install_action
    {
        println!("\n## Npx Install (GSD)");
        println!("\nRun the following command:");
        let scope_flag = match scope {
            Scope::Global => "--global",
            Scope::Project => "--local",
        };
        println!(
            "  npx {} {} --<tool> {}",
            package,
            args.join(" "),
            scope_flag
        );
        if let Some(flag) = uninstall_flag {
            println!("\nTo uninstall:");
            println!(
                "  npx {} {} {} --<tool> {}",
                package,
                args.join(" "),
                flag,
                scope_flag
            );
        }
    }
}

fn remove_skill(id: String, global: bool, agent_mode: bool, _verbose: bool) -> Result<()> {
    let scope = if global {
        Scope::Global
    } else {
        Scope::Project
    };

    let global_config = GlobalConfig::load();
    let project_config_path = Path::new(".rulesify.toml");

    if global {
        // `get_tools_for_skill` returns only tools with direct entries
        // (not covered tools), which is the correct set for physical uninstall.
        let tools = global_config.get_tools_for_skill(&id);
        if tools.is_empty() {
            println!("'{}' is not installed globally.", id);
            return Ok(());
        }

        if agent_mode {
            println!("{}", generate_uninstall_instructions(&id, &tools, scope));
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

        if agent_mode {
            println!(
                "{}",
                generate_uninstall_instructions(&id, &project_config.tools, scope)
            );
            return Ok(());
        }

        // Resolve Pi coverage: only delete physical installs.
        // Covered tools (e.g. Pi) have no files to clean up.
        let (physical_tools, _) = resolve_pi_coverage(&project_config.tools);

        let results = uninstall_skill(&id, &physical_tools, scope);

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

async fn update_directory_registry(agent_mode: bool, force: bool, verbose: bool) -> Result<()> {
    // 1. Check local registry.toml date
    let local_path = Path::new("registry.toml");
    let local_updated = if local_path.exists() {
        let content = std::fs::read_to_string(local_path)?;
        let local: Registry = toml::from_str(&content)?;
        local.updated
    } else {
        String::new()
    };

    // 2. Fetch remote registry
    println!("Fetching remote registry...");
    let registry = fetch_registry().await?;

    // 3. Compare dates — skip if local is already current
    if needs_registry_update(force, &local_updated, &registry.updated) {
        if force {
            println!("Force updating local registry...");
        } else {
            println!(
                "Updating local registry ({} \u{2192} {})...",
                local_updated, registry.updated
            );
        }

        let content = toml::to_string_pretty(&registry)?;
        std::fs::write(local_path, content)?;
        println!("Local registry updated ({} skills)", registry.skills.len());
    }

    // 4. Save to local cache (always, so installed-skill update can use it)
    let cache = RegistryCache::new(Path::new("."));
    cache.save(&registry)?;

    if verbose {
        println!("Updated date: {}", registry.updated);
    }

    if agent_mode {
        println!("\nTo update installed skills, run:");
        println!("  rulesify skill update");
        return Ok(());
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

    let archive_cache = ArchiveCache::new();
    let client = GitHubClient::new();

    for (tool, _id, skill) in &global_updated {
        println!("\nUpdating '{}' [{}] (global)...", skill.name, tool);

        let results = match &skill.install_action {
            Some(InstallAction::Npx {
                package,
                args,
                uninstall_flag,
            }) => execute_npx_install(
                package,
                args,
                uninstall_flag.as_deref(),
                std::slice::from_ref(&tool),
                Scope::Global,
            )?,
            Some(InstallAction::MegaSkillCopy {
                source_folder,
                dest_name,
            }) => {
                install_mega_skill(
                    skill,
                    source_folder,
                    dest_name,
                    std::slice::from_ref(&tool),
                    Scope::Global,
                    &client,
                    &archive_cache,
                )
                .await?
            }
            _ => {
                install_skill(
                    skill,
                    std::slice::from_ref(&tool),
                    Scope::Global,
                    &client,
                    &archive_cache,
                )
                .await?
            }
        };
        print_install_summary(&results, &skill.name);
    }

    if !project_updated.is_empty() {
        let Some(ref config) = project_config else {
            return Err(RulesifyError::ConfigNotFound.into());
        };
        let tools = config.tools.clone();
        let (physical_tools, _) = resolve_pi_coverage(&tools);
        for (_id, skill) in &project_updated {
            println!("\nUpdating '{}' (project)...", skill.name);

            let results = match &skill.install_action {
                Some(InstallAction::Npx {
                    package,
                    args,
                    uninstall_flag,
                }) => execute_npx_install(
                    package,
                    args,
                    uninstall_flag.as_deref(),
                    &physical_tools,
                    Scope::Project,
                )?,
                Some(InstallAction::MegaSkillCopy {
                    source_folder,
                    dest_name,
                }) => {
                    install_mega_skill(
                        skill,
                        source_folder,
                        dest_name,
                        &physical_tools,
                        Scope::Project,
                        &client,
                        &archive_cache,
                    )
                    .await?
                }
                _ => {
                    install_skill(
                        skill,
                        &physical_tools,
                        Scope::Project,
                        &client,
                        &archive_cache,
                    )
                    .await?
                }
            };
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
    // Prefer built-in registry so local changes are immediately visible.
    // Run `rulesify skill update` to explicitly sync the latest remote.
    load_builtin()
}

fn needs_registry_update(force: bool, local_updated: &str, remote_updated: &str) -> bool {
    if force {
        return true;
    }
    if local_updated.is_empty() {
        return true;
    }
    remote_updated > local_updated
}

fn load_project_config(path: &Path) -> Result<Option<ProjectConfig>> {
    ProjectConfig::reconcile_and_load(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_update_when_remote_is_newer() {
        assert!(needs_registry_update(false, "2026-01-01", "2026-06-15"));
    }

    #[test]
    fn test_needs_update_when_local_is_current() {
        assert!(!needs_registry_update(false, "2026-06-15", "2026-06-15"));
    }

    #[test]
    fn test_needs_update_when_local_is_newer() {
        assert!(!needs_registry_update(false, "2026-07-20", "2026-06-15"));
    }

    #[test]
    fn test_needs_update_when_no_local_file() {
        assert!(needs_registry_update(false, "", "2026-06-15"));
    }

    #[test]
    fn test_force_overrides_freshness_check() {
        assert!(needs_registry_update(true, "2026-07-20", "2026-06-15"));
        assert!(needs_registry_update(true, "2026-06-15", "2026-06-15"));
        assert!(needs_registry_update(true, "", "2026-06-15"));
    }
}
