use crate::fetcher::ArchiveCache;
use crate::installer::executor::{
    install_mega_skill, install_skill, parse_source_url, uninstall_skill,
};
use crate::models::{InstallAction, Scope, Skill};
use crate::registry::github::GitHubClient;
use serial_test::serial;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_parse_source_url_valid() {
    let url = "https://github.com/openai/skills/tree/main/skills/.curated/render-deploy";
    let source = parse_source_url(url).unwrap();

    assert_eq!(source.owner, "openai");
    assert_eq!(source.repo, "skills");
    assert_eq!(source.branch, "main");
    assert_eq!(source.folder, "skills/.curated/render-deploy");
}

#[test]
fn test_parse_source_url_invalid_format() {
    let url = "https://github.com/openai/skills";
    let result = parse_source_url(url);
    assert!(result.is_err());
}

#[test]
fn test_parse_source_url_missing_owner_repo() {
    let url = "https://github.com/tree/main/skills";
    let result = parse_source_url(url);
    assert!(result.is_err());
}

#[test]
fn test_parse_source_url_with_subfolder() {
    let url = "https://github.com/anthropics/skills/tree/v2/skills/brainstorming";
    let source = parse_source_url(url).unwrap();

    assert_eq!(source.owner, "anthropics");
    assert_eq!(source.repo, "skills");
    assert_eq!(source.branch, "v2");
    assert_eq!(source.folder, "skills/brainstorming");
}

#[test]
#[serial]
fn test_uninstall_skill_deletes_folders() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let skill_name = "test-skill";
    let tools = vec!["claude-code".to_string(), "codex".to_string()];

    let claude_path = base_path.join(".claude/skills").join(skill_name);
    let codex_path = base_path.join(".agents/skills").join(skill_name);

    fs::create_dir_all(&claude_path).unwrap();
    fs::create_dir_all(&codex_path).unwrap();

    fs::write(claude_path.join("SKILL.md"), "test content").unwrap();
    fs::write(codex_path.join("SKILL.md"), "test content").unwrap();

    assert!(claude_path.exists());
    assert!(codex_path.exists());

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let results = uninstall_skill(skill_name, &tools, Scope::Project);

    let claude_exists = claude_path.exists();
    let codex_exists = codex_path.exists();

    std::env::set_current_dir(&original_dir).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results[0].folder_deleted);
    assert!(results[1].folder_deleted);

    assert!(!claude_exists);
    assert!(!codex_exists);
}

#[test]
#[serial]
fn test_uninstall_skill_missing_folder() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let skill_name = "nonexistent-skill";
    let tools = vec!["claude-code".to_string()];

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let results = uninstall_skill(skill_name, &tools, Scope::Project);

    std::env::set_current_dir(&original_dir).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].folder_deleted);
    assert!(results[0].error.is_none());
}

#[test]
#[serial]
fn test_uninstall_skill_multiple_tools() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let skill_name = "multi-tool-skill";
    let tools = vec![
        "claude-code".to_string(),
        "codex".to_string(),
        "cursor".to_string(),
    ];

    let claude_path = base_path.join(".claude/skills").join(skill_name);
    let codex_path = base_path.join(".agents/skills").join(skill_name);
    let cursor_path = base_path.join(".cursor/skills").join(skill_name);

    fs::create_dir_all(&claude_path).unwrap();
    fs::create_dir_all(&codex_path).unwrap();
    fs::create_dir_all(&cursor_path).unwrap();

    fs::write(claude_path.join("SKILL.md"), "test").unwrap();
    fs::write(codex_path.join("SKILL.md"), "test").unwrap();
    fs::write(cursor_path.join("SKILL.md"), "test").unwrap();

    assert!(claude_path.exists());
    assert!(codex_path.exists());
    assert!(cursor_path.exists());

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let results = uninstall_skill(skill_name, &tools, Scope::Project);

    let claude_exists = claude_path.exists();
    let codex_exists = codex_path.exists();
    let cursor_exists = cursor_path.exists();

    std::env::set_current_dir(&original_dir).unwrap();

    assert_eq!(results.len(), 3);
    for r in &results {
        assert!(r.folder_deleted);
    }

    assert!(!claude_exists);
    assert!(!codex_exists);
    assert!(!cursor_exists);
}

fn make_normal_skill(name: &str, source_url: &str) -> Skill {
    Skill {
        name: name.to_string(),
        description: "Test skill".to_string(),
        source_url: source_url.to_string(),
        stars: 100,
        commit_sha: "test123".to_string(),
        context_size: 1000,
        domain: "development".to_string(),
        last_updated: "2026-04-21".to_string(),
        tags: vec!["testing".to_string()],
        install_action: None,
        score: Some(80.0),
        is_mega_skill: false,
    }
}

fn make_mega_skill(name: &str, source_url: &str, source_folder: &str, dest_name: &str) -> Skill {
    Skill {
        name: name.to_string(),
        description: "Test mega-skill".to_string(),
        source_url: source_url.to_string(),
        stars: 1000,
        commit_sha: "mega123".to_string(),
        context_size: 0,
        domain: "development".to_string(),
        last_updated: "2026-04-21".to_string(),
        tags: vec!["mega-skill".to_string()],
        install_action: Some(InstallAction::mega_skill_copy(source_folder, dest_name)),
        score: Some(90.0),
        is_mega_skill: true,
    }
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_skill_real_fetch() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_normal_skill(
        "brainstorming",
        "https://github.com/anthropics/skills/tree/main/skills/brainstorming",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string()];

    let results = install_skill(&skill, &tools, Scope::Project, &client, &cache)
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert!(results[0].files_created > 0);

    let skill_path = base_path.join(".claude/skills/brainstorming");
    assert!(skill_path.exists());
    assert!(skill_path.join("SKILL.md").exists());

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_skill_creates_skill_md() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_normal_skill(
        "xlsx",
        "https://github.com/anthropics/skills/tree/main/skills/xlsx",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string()];

    let _results = install_skill(&skill, &tools, Scope::Project, &client, &cache)
        .await
        .unwrap();

    let skill_path = base_path.join(".claude/skills/xlsx");
    assert!(skill_path.exists());

    let skill_md = skill_path.join("SKILL.md");
    assert!(skill_md.exists());

    let content = fs::read_to_string(&skill_md).unwrap();
    assert!(content.contains("---"));
    assert!(content.contains("name:"));

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_skill_multiple_tools_real() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_normal_skill(
        "doc",
        "https://github.com/openai/skills/tree/main/skills/.curated/doc",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec![
        "claude-code".to_string(),
        "codex".to_string(),
        "cursor".to_string(),
    ];

    let results = install_skill(&skill, &tools, Scope::Project, &client, &cache)
        .await
        .unwrap();

    assert_eq!(results.len(), 3);
    for r in &results {
        assert!(r.success);
    }

    assert!(base_path.join(".claude/skills/doc").exists());
    assert!(base_path.join(".agents/skills/doc").exists());
    assert!(base_path.join(".cursor/skills/doc").exists());

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_skill_global_scope() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_normal_skill(
        "skill-creator",
        "https://github.com/anthropics/skills/tree/main/skills/skill-creator",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string()];

    let results = install_skill(&skill, &tools, Scope::Global, &client, &cache)
        .await
        .unwrap();

    assert!(results[0].success);

    let global_skill_path = dirs::home_dir()
        .unwrap()
        .join(".claude")
        .join("skills")
        .join("skill-creator");
    assert!(global_skill_path.exists());

    uninstall_skill("skill-creator", &tools, Scope::Global);

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_mega_skill_real_fetch() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_mega_skill(
        "superpowers",
        "https://github.com/obra/superpowers/tree/main/",
        "skills",
        "superpowers",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string()];

    let results = install_mega_skill(
        &skill,
        "skills",
        "superpowers",
        &tools,
        Scope::Project,
        &client,
        &cache,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert!(results[0].files_created > 0);

    let skill_path = base_path.join(".claude/skills/superpowers");
    assert!(skill_path.exists());
    assert!(skill_path.is_dir());

    let entries: Vec<_> = fs::read_dir(&skill_path)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(entries.len() > 1);

    for entry in &entries {
        if entry.path().is_dir() {
            assert!(entry.path().join("SKILL.md").exists());
        }
    }

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_install_mega_skill_multiple_tools_real() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_mega_skill(
        "superpowers",
        "https://github.com/obra/superpowers/tree/main/",
        "skills",
        "superpowers",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string(), "codex".to_string()];

    let results = install_mega_skill(
        &skill,
        "skills",
        "superpowers",
        &tools,
        Scope::Project,
        &client,
        &cache,
    )
    .await
    .unwrap();

    assert_eq!(results.len(), 2);
    for r in &results {
        assert!(r.success);
    }

    let claude_path = base_path.join(".claude/skills/superpowers");
    let codex_path = base_path.join(".agents/skills/superpowers");

    assert!(claude_path.exists());
    assert!(codex_path.exists());

    std::env::set_current_dir(&original_dir).unwrap();
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_uninstall_mega_skill_after_install() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_mega_skill(
        "superpowers",
        "https://github.com/obra/superpowers/tree/main/",
        "skills",
        "superpowers",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string(), "codex".to_string()];

    let results = install_mega_skill(
        &skill,
        "skills",
        "superpowers",
        &tools,
        Scope::Project,
        &client,
        &cache,
    )
    .await
    .unwrap();

    for r in &results {
        assert!(r.success);
    }

    let claude_path = base_path.join(".claude/skills/superpowers");
    let codex_path = base_path.join(".agents/skills/superpowers");

    assert!(claude_path.exists());
    assert!(codex_path.exists());

    let uninstall_results = uninstall_skill("superpowers", &tools, Scope::Project);

    let claude_exists = claude_path.exists();
    let codex_exists = codex_path.exists();

    std::env::set_current_dir(&original_dir).unwrap();

    assert_eq!(uninstall_results.len(), 2);
    for r in &uninstall_results {
        assert!(r.folder_deleted);
        assert!(r.error.is_none());
    }

    assert!(!claude_exists);
    assert!(!codex_exists);
}

#[tokio::test]
#[ignore]
#[serial]
async fn test_uninstall_mega_skill_global_scope() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    let original_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(base_path).unwrap();

    let skill = make_mega_skill(
        "superpowers",
        "https://github.com/obra/superpowers/tree/main/",
        "skills",
        "superpowers",
    );

    let client = GitHubClient::new();
    let cache = ArchiveCache::new();
    let tools = vec!["claude-code".to_string()];

    let results = install_mega_skill(
        &skill,
        "skills",
        "superpowers",
        &tools,
        Scope::Global,
        &client,
        &cache,
    )
    .await
    .unwrap();

    assert!(results[0].success);

    let global_path = dirs::home_dir()
        .unwrap()
        .join(".claude")
        .join("skills")
        .join("superpowers");
    assert!(global_path.exists());

    uninstall_skill("superpowers", &tools, Scope::Global);

    std::env::set_current_dir(&original_dir).unwrap();

    assert!(!global_path.exists());
}
