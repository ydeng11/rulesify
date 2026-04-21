use crate::fetcher::ArchiveCache;
use crate::installer::tool_paths::get_skill_folder;
use crate::models::{Scope, Skill};
use crate::registry::github::GitHubClient;
use crate::utils::{Result, RulesifyError};
use std::path::PathBuf;
use std::process::Command;

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
    _client: &GitHubClient,
    cache: &ArchiveCache,
) -> Result<Vec<InstallResult>> {
    let source = parse_source_url(&skill.source_url)?;

    let extracted_folder = cache.get_extracted_folder(&source).await?;

    let entries: Vec<_> = std::fs::read_dir(&extracted_folder)
        .map_err(|e| RulesifyError::SkillParse(format!("Failed to read extracted folder: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();

    let mut results = Vec::new();

    for tool in tools {
        let skill_folder = get_skill_folder(tool, scope.clone(), &skill.name);
        let result = install_for_tool(&extracted_folder, &entries, &skill_folder, tool.clone());
        results.push(result);
    }

    Ok(results)
}

pub async fn install_mega_skill(
    skill: &Skill,
    source_folder: &str,
    dest_name: &str,
    tools: &[String],
    scope: Scope,
    _client: &GitHubClient,
    cache: &ArchiveCache,
) -> Result<Vec<InstallResult>> {
    let source = parse_source_url(&skill.source_url)?;

    let extracted_root = cache.get_extracted_folder(&source).await?;

    let source_path = extracted_root.join(source_folder);

    if !source_path.exists() {
        return Err(RulesifyError::SkillParse(format!(
            "Source folder '{}' not found in archive",
            source_folder
        ))
        .into());
    }

    let entries: Vec<_> = std::fs::read_dir(&source_path)
        .map_err(|e| RulesifyError::SkillParse(format!("Failed to read source folder: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();

    let mut results = Vec::new();

    for tool in tools {
        let skill_folder = get_skill_folder(tool, scope.clone(), dest_name);
        let result = install_for_tool(&source_path, &entries, &skill_folder, tool.clone());
        results.push(result);
    }

    Ok(results)
}

fn install_for_tool(
    extracted_folder: &PathBuf,
    entries: &[std::fs::DirEntry],
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

    for entry in entries {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let source_path = extracted_folder.join(&file_name);
        let target_path = skill_folder.join(&file_name);

        if entry.path().is_dir() {
            if let Err(e) = copy_dir_all(&source_path, &target_path) {
                errors.push(format!("{}: {}", file_name, e));
            } else {
                files_created += 1;
            }
        } else {
            if let Err(e) = std::fs::copy(&source_path, &target_path) {
                errors.push(format!("{}: {}", file_name, e));
            } else {
                files_created += 1;
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

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn execute_npx_install(
    package: &str,
    args: &[String],
    _uninstall_flag: Option<&str>,
    tools: &[String],
    scope: Scope,
) -> Result<Vec<InstallResult>> {
    let mut results = Vec::new();

    for tool in tools {
        let tool_flag = match tool.as_str() {
            "claude-code" => "--claude",
            "opencode" => "--opencode",
            "cursor" => "--cursor",
            "codex" => "--codex",
            "pi" => "--pi",
            _ => "",
        };

        let scope_flag = match scope {
            Scope::Global => "--global",
            Scope::Project => "--local",
        };

        let mut full_args = vec![package];
        full_args.extend(args.iter().map(|s| s.as_str()));
        if !tool_flag.is_empty() {
            full_args.push(tool_flag);
        }
        full_args.push(scope_flag);

        let output = Command::new("npx").args(&full_args).output();

        match output {
            Ok(o) if o.status.success() => {
                results.push(InstallResult {
                    tool: tool.clone(),
                    files_created: 0,
                    success: true,
                    error: None,
                });
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                results.push(InstallResult {
                    tool: tool.clone(),
                    files_created: 0,
                    success: false,
                    error: Some(stderr),
                });
            }
            Err(e) => {
                results.push(InstallResult {
                    tool: tool.clone(),
                    files_created: 0,
                    success: false,
                    error: Some(format!("Failed to run npx: {}", e)),
                });
            }
        }
    }

    Ok(results)
}

pub fn execute_npx_uninstall(
    package: &str,
    args: &[String],
    uninstall_flag: Option<&str>,
    tools: &[String],
    scope: Scope,
) -> Vec<UninstallResult> {
    let uninstall_flag = uninstall_flag.unwrap_or("--uninstall");

    let mut results = Vec::new();

    for tool in tools {
        let tool_flag = match tool.as_str() {
            "claude-code" => "--claude",
            "opencode" => "--opencode",
            "cursor" => "--cursor",
            "codex" => "--codex",
            "pi" => "--pi",
            _ => "",
        };

        let scope_flag = match scope {
            Scope::Global => "--global",
            Scope::Project => "--local",
        };

        let mut full_args = vec![package];
        full_args.extend(args.iter().map(|s| s.as_str()));
        full_args.push(uninstall_flag);
        if !tool_flag.is_empty() {
            full_args.push(tool_flag);
        }
        full_args.push(scope_flag);

        let output = Command::new("npx").args(&full_args).output();

        match output {
            Ok(o) if o.status.success() => {
                results.push(UninstallResult {
                    tool: tool.clone(),
                    folder_deleted: true,
                    error: None,
                });
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                results.push(UninstallResult {
                    tool: tool.clone(),
                    folder_deleted: false,
                    error: Some(stderr),
                });
            }
            Err(e) => {
                results.push(UninstallResult {
                    tool: tool.clone(),
                    folder_deleted: false,
                    error: Some(format!("Failed to run npx: {}", e)),
                });
            }
        }
    }

    results
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
