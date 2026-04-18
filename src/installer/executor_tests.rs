use crate::installer::executor::{parse_source_url, uninstall_skill};
use crate::models::Scope;
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
