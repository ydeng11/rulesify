use crate::fetcher::ArchiveCache;
use crate::installer::tool_paths::get_skill_folder;
use crate::models::{Scope, Skill};
use crate::registry::github::GitHubClient;
use crate::registry::parser::SkillParser;
use crate::utils::{Result, RulesifyError};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug)]
pub struct InstallResult {
    pub tool: String,
    pub files_created: usize,
    pub success: bool,
    pub error: Option<String>,
    pub warning: Option<String>,
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
    pub archive_ref: String,
    pub folder: String,
}

impl SkillSource {
    pub fn archive_ref(&self) -> &str {
        if self.archive_ref.trim().is_empty() {
            &self.branch
        } else {
            &self.archive_ref
        }
    }

    fn use_commit_sha(&mut self, commit_sha: &str) {
        let commit_sha = commit_sha.trim();
        if !commit_sha.is_empty() {
            self.archive_ref = commit_sha.to_string();
        }
    }
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
        archive_ref: branch.clone(),
        branch,
        folder,
    })
}

pub async fn install_skill<T: AsRef<str>>(
    skill: &Skill,
    tools: &[T],
    scope: Scope,
    _client: &GitHubClient,
    cache: &ArchiveCache,
) -> Result<Vec<InstallResult>> {
    let mut source = parse_source_url(&skill.source_url)?;
    source.use_commit_sha(&skill.commit_sha);

    let resolved = resolve_skill_folder(skill, &source, cache).await?;

    let entries: Vec<_> = std::fs::read_dir(&resolved.path)
        .map_err(|e| RulesifyError::SkillParse(format!("Failed to read extracted folder: {}", e)))?
        .filter_map(|e| e.ok())
        .collect();

    let mut results = Vec::new();

    for tool in tools {
        let skill_folder = get_skill_folder(tool.as_ref(), scope, &skill.name);
        let result = install_for_tool(
            &resolved.path,
            &entries,
            &skill_folder,
            tool.as_ref(),
            resolved.warning.clone(),
        );
        results.push(result);
    }

    Ok(results)
}

#[derive(Debug)]
pub(crate) struct ResolvedSkillFolder {
    pub(crate) path: PathBuf,
    pub(crate) warning: Option<String>,
}

pub(crate) async fn resolve_skill_folder(
    skill: &Skill,
    source: &SkillSource,
    cache: &ArchiveCache,
) -> Result<ResolvedSkillFolder> {
    match cache.get_extracted_folder(source).await {
        Ok(path) => Ok(ResolvedSkillFolder {
            path,
            warning: None,
        }),
        Err(original_error) => {
            let repo_root = cache.get_extracted_repo_root(source).await?;
            let matches = find_skill_folders_by_name(&repo_root, &skill.name)?;

            match matches.as_slice() {
                [] => Err(original_error),
                [path] => {
                    let resolved_folder = relative_path(&repo_root, path);
                    Ok(ResolvedSkillFolder {
                        path: path.clone(),
                        warning: Some(format!(
                            "Source path moved; installing '{}' from {} instead of {}",
                            skill.name, resolved_folder, source.folder
                        )),
                    })
                }
                _ => Err(RulesifyError::SkillParse(format!(
                    "Multiple folders named '{}' found in archive: {}",
                    skill.name,
                    format_candidate_paths(&repo_root, &matches)
                ))
                .into()),
            }
        }
    }
}

#[cfg(test)]
pub(crate) fn find_skill_folder_by_name(
    repo_root: &Path,
    skill_name: &str,
) -> Result<Option<PathBuf>> {
    let matches = find_skill_folders_by_name(repo_root, skill_name)?;

    match matches.as_slice() {
        [] => Ok(None),
        [path] => Ok(Some(path.clone())),
        _ => Err(RulesifyError::SkillParse(format!(
            "Multiple folders named '{}' found in archive: {}",
            skill_name,
            format_candidate_paths(repo_root, &matches)
        ))
        .into()),
    }
}

pub(crate) fn find_skill_folders_by_name(
    repo_root: &Path,
    skill_name: &str,
) -> Result<Vec<PathBuf>> {
    let mut matches = Vec::new();

    for entry in WalkDir::new(repo_root)
        .into_iter()
        .filter_entry(|entry| !is_hidden_or_build_dir(entry.path()))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file() && entry.file_name() == "SKILL.md")
    {
        let content = std::fs::read_to_string(entry.path()).map_err(|e| {
            RulesifyError::SkillParse(format!("Failed to read skill metadata: {}", e))
        })?;

        match SkillParser::parse(&content) {
            Ok(parsed) if parsed.name == skill_name => {
                if let Some(parent) = entry.path().parent() {
                    matches.push(parent.to_path_buf());
                }
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }

    Ok(matches)
}

fn relative_path(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn format_candidate_paths(repo_root: &Path, paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| relative_path(repo_root, path))
        .collect::<Vec<_>>()
        .join(", ")
}

fn is_hidden_or_build_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| matches!(name, ".git" | "target" | "node_modules"))
        .unwrap_or(false)
}

pub async fn install_mega_skill<T: AsRef<str>>(
    skill: &Skill,
    source_folder: &str,
    dest_name: &str,
    tools: &[T],
    scope: Scope,
    _client: &GitHubClient,
    cache: &ArchiveCache,
) -> Result<Vec<InstallResult>> {
    let mut source = parse_source_url(&skill.source_url)?;
    source.use_commit_sha(&skill.commit_sha);

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
        let skill_folder = get_skill_folder(tool.as_ref(), scope, dest_name);
        let result = install_for_tool(&source_path, &entries, &skill_folder, tool.as_ref(), None);
        results.push(result);
    }

    Ok(results)
}

fn install_for_tool(
    extracted_folder: &Path,
    entries: &[std::fs::DirEntry],
    skill_folder: &Path,
    tool: &str,
    warning: Option<String>,
) -> InstallResult {
    if skill_folder.exists() {
        if let Err(e) = std::fs::remove_dir_all(skill_folder) {
            return InstallResult {
                tool: tool.to_string(),
                files_created: 0,
                success: false,
                error: Some(format!("Failed to clear existing folder: {}", e)),
                warning,
            };
        }
    }

    if let Err(e) = std::fs::create_dir_all(skill_folder) {
        return InstallResult {
            tool: tool.to_string(),
            files_created: 0,
            success: false,
            error: Some(format!("Failed to create folder: {}", e)),
            warning,
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
        tool: tool.to_string(),
        files_created,
        success: errors.is_empty(),
        error: if errors.is_empty() {
            None
        } else {
            Some(errors.join("; "))
        },
        warning,
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

pub fn execute_npx_install<T: AsRef<str>>(
    package: &str,
    args: &[String],
    _uninstall_flag: Option<&str>,
    tools: &[T],
    scope: Scope,
) -> Result<Vec<InstallResult>> {
    let mut results = Vec::new();

    for tool in tools {
        let tool_flag = match tool.as_ref() {
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
                    tool: tool.as_ref().to_string(),
                    files_created: 0,
                    success: true,
                    error: None,
                    warning: None,
                });
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).to_string();
                results.push(InstallResult {
                    tool: tool.as_ref().to_string(),
                    files_created: 0,
                    success: false,
                    error: Some(stderr),
                    warning: None,
                });
            }
            Err(e) => {
                results.push(InstallResult {
                    tool: tool.as_ref().to_string(),
                    files_created: 0,
                    success: false,
                    error: Some(format!("Failed to run npx: {}", e)),
                    warning: None,
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
            let skill_folder = get_skill_folder(tool, scope, skill_name);
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
    let warnings: BTreeSet<&str> = results
        .iter()
        .filter_map(|r| r.warning.as_deref())
        .collect();

    for warning in warnings {
        println!("  ! {}", warning);
    }

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
