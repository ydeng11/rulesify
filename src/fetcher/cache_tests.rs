use crate::fetcher::cache::{find_extracted_repo_root, get_cache_key};
use crate::installer::executor::SkillSource;
use std::fs;
use tempfile::TempDir;

fn make_source(
    owner: &str,
    repo: &str,
    branch: &str,
    archive_ref: &str,
    folder: &str,
) -> SkillSource {
    SkillSource {
        owner: owner.to_string(),
        repo: repo.to_string(),
        branch: branch.to_string(),
        archive_ref: archive_ref.to_string(),
        folder: folder.to_string(),
    }
}

#[test]
fn test_cache_key_generation() {
    let source = make_source(
        "obra",
        "superpowers",
        "main",
        "main",
        "skills/test-driven-development",
    );

    let key = get_cache_key(&source);
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_different_sources_different_keys() {
    let source1 = make_source(
        "obra",
        "superpowers",
        "main",
        "main",
        "skills/test-driven-development",
    );
    let source2 = make_source(
        "pbakaus",
        "impeccable",
        "main",
        "main",
        ".agents/skills/audit",
    );

    let key1 = get_cache_key(&source1);
    let key2 = get_cache_key(&source2);

    assert_ne!(key1, key2);
}

#[test]
fn test_same_repo_same_key() {
    let source1 = make_source(
        "obra",
        "superpowers",
        "main",
        "main",
        "skills/test-driven-development",
    );
    let source2 = make_source(
        "obra",
        "superpowers",
        "main",
        "main",
        "skills/systematic-debugging",
    );

    let key1 = get_cache_key(&source1);
    let key2 = get_cache_key(&source2);

    assert_eq!(key1, key2);
}

#[test]
fn test_same_repo_different_commits_different_keys() {
    let source1 = make_source("obra", "superpowers", "main", "abc123", "skills/tdd");
    let source2 = make_source("obra", "superpowers", "main", "def456", "skills/tdd");

    assert_ne!(get_cache_key(&source1), get_cache_key(&source2));
}

#[test]
fn test_empty_archive_ref_falls_back_to_branch_for_cache_key() {
    let source1 = make_source("obra", "superpowers", "main", "", "skills/tdd");
    let source2 = make_source("obra", "superpowers", "main", "main", "skills/tdd");

    assert_eq!(get_cache_key(&source1), get_cache_key(&source2));
}

#[test]
fn test_find_extracted_repo_root_branch_style() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("rulesify-main");
    fs::create_dir(&root).unwrap();

    assert_eq!(find_extracted_repo_root(temp_dir.path()).unwrap(), root);
}

#[test]
fn test_find_extracted_repo_root_commit_style() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path().join("rulesify-62f43a1");
    fs::create_dir(&root).unwrap();

    assert_eq!(find_extracted_repo_root(temp_dir.path()).unwrap(), root);
}

#[test]
fn test_find_extracted_repo_root_rejects_multiple_roots() {
    let temp_dir = TempDir::new().unwrap();
    fs::create_dir(temp_dir.path().join("one")).unwrap();
    fs::create_dir(temp_dir.path().join("two")).unwrap();

    let err = find_extracted_repo_root(temp_dir.path()).unwrap_err();
    assert!(err.to_string().contains("Expected one extracted repo root"));
}
