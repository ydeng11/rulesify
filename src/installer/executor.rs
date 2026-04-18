use crate::installer::tool_paths::get_skill_folder;
use crate::models::{InstallAction, Scope, Skill};
use crate::registry::github::GitHubClient;
use crate::utils::{Result, RulesifyError};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug)]
pub struct InstallResult {
    pub tool: String,
    pub files_created: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct UninstallResult {
    pub tool: String,
    pub folder_deleted: bool,
    pub error: Option<String>,
}

pub struct SkillSource {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub folder: String,
}

pub fn parse_source_url(source_url: &str) -> Result<SkillSource> {
    let url = source_url.trim_start_matches("https://github.com/");

    let parts: Vec<&str> = url.split("/tree/").collect();
    if parts.len() != 2 {
        return Err(RulesifyError::SkillParse(format!(
            "Invalid source URL format: {}",
            source_url
        ))
        .into());
    }

    let base_parts: Vec<&str> = parts[0].split('/').collect();
    if base_parts.len() < 2 {
        return Err(RulesifyError::SkillParse("Missing owner/repo in URL".into()).into());
    }

    let owner = base_parts[0].to_string();
    let repo = base_parts[1].to_string();

    let path_parts: Vec<&str> = parts[1].split('/').collect();
    let branch = path_parts.first().unwrap_or(&"main").to_string();
    let folder = path_parts[1..].join("/");

    Ok(SkillSource {
        owner,
        repo,
        branch,
        folder,
    })
}

pub async fn install_skill(
    skill: &Skill,
    tools: &[String],
    scope: Scope,
    client: &GitHubClient,
) -> Result<Vec<InstallResult>> {
    let source = parse_source_url(&skill.source_url)?;

    let folder_path = skill
        .install_action
        .as_ref()
        .and_then(|a| match a {
            InstallAction::Copy { folder } => Some(folder.clone()),
            InstallAction::Command { .. } => None,
        })
        .unwrap_or_else(|| source.folder.clone());

    let entries = client
        .list_folder(&source.owner, &source.repo, &folder_path)
        .await?;

    let files: Vec<_> = entries
        .iter()
        .filter(|e| e.content_type == "file")
        .collect();

    let mut results = Vec::new();

    for tool in tools {
        let skill_folder = get_skill_folder(tool, scope.clone(), &skill.name);
        let result = install_for_tool(
            client,
            &source,
            &folder_path,
            &files,
            &skill_folder,
            tool.clone(),
        )
        .await;
        results.push(result);
    }

    Ok(results)
}

async fn install_for_tool(
    client: &GitHubClient,
    source: &SkillSource,
    folder_path: &str,
    files: &[&crate::registry::github::ContentEntry],
    skill_folder: &PathBuf,
    tool: String,
) -> InstallResult {
    if skill_folder.exists() {
        if let Err(e) = std::fs::remove_dir_all(skill_folder) {
            return InstallResult {
                tool,
                files_created: 0,
                success: false,
                error: Some(format!("Failed to clear existing folder: {}", e)),
            };
        }
    }

    if let Err(e) = std::fs::create_dir_all(skill_folder) {
        return InstallResult {
            tool,
            files_created: 0,
            success: false,
            error: Some(format!("Failed to create folder: {}", e)),
        };
    }

    let mut files_created = 0;
    let mut errors = Vec::new();

    for file_entry in files {
        let file_path = format!("{}/{}", folder_path, file_entry.name);
        let local_path = skill_folder.join(&file_entry.name);

        match client
            .fetch_file_raw(&source.owner, &source.repo, &file_path)
            .await
        {
            Ok(content) => {
                if let Err(e) = std::fs::write(&local_path, &content) {
                    errors.push(format!("{}: {}", file_entry.name, e));
                } else {
                    files_created += 1;
                }
            }
            Err(e) => {
                errors.push(format!("{}: fetch failed - {}", file_entry.name, e));
            }
        }
    }

    InstallResult {
        tool,
        files_created,
        success: errors.is_empty(),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
    }
}

pub fn uninstall_skill(skill_name: &str, tools: &[String], scope: Scope) -> Vec<UninstallResult> {
    tools
        .iter()
        .map(|tool| {
            let skill_folder = get_skill_folder(tool, scope.clone(), skill_name);
            uninstall_for_tool(skill_folder, tool.clone())
        })
        .collect()
}

fn uninstall_for_tool(skill_folder: PathBuf, tool: String) -> UninstallResult {
    if !skill_folder.exists() {
        return UninstallResult {
            tool,
            folder_deleted: true,
            error: None,
        };
    }

    match std::fs::remove_dir_all(&skill_folder) {
        Ok(_) => UninstallResult {
            tool,
            folder_deleted: true,
            error: None,
        },
        Err(e) => UninstallResult {
            tool,
            folder_deleted: false,
            error: Some(e.to_string()),
        },
    }
}

pub fn prompt_confirm(message: &str) -> bool {
    print!("{} [y/N]: ", message);
    io::stdout().flush().ok();

    let stdin = io::stdin();
    let mut input = String::new();
    stdin.lock().read_line(&mut input).ok();

    input.trim().eq_ignore_ascii_case("y")
}

pub fn print_install_summary(results: &[InstallResult], skill_name: &str) {
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    if failed == 0 {
        println!(
            "Installed '{}' to {} tools ({} files each)",
            skill_name,
            successful,
            results.first().map(|r| r.files_created).unwrap_or(0)
        );
    } else {
        println!("Installed '{}' with issues:", skill_name);
        for r in results {
            if r.success {
                println!("  ✓ {}: {} files", r.tool, r.files_created);
            } else {
                println!(
                    "  ✗ {}: {}",
                    r.tool,
                    r.error.as_deref().unwrap_or("unknown error")
                );
            }
        }
    }
}

pub fn print_uninstall_summary(results: &[UninstallResult], skill_name: &str) {
    let successful = results.iter().filter(|r| r.folder_deleted).count();
    let failed = results.len() - successful;

    if failed == 0 {
        println!("Removed '{}' from {} tools", skill_name, successful);
    } else {
        println!("Removed '{}' with issues:", skill_name);
        for r in results {
            if r.folder_deleted {
                println!("  ✓ {}", r.tool);
            } else {
                println!(
                    "  ✗ {}: {}",
                    r.tool,
                    r.error.as_deref().unwrap_or("unknown error")
                );
            }
        }
    }
}
