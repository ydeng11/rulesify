use crate::fetcher::cache::get_cache_key;
use crate::installer::executor::SkillSource;

#[test]
fn test_cache_key_generation() {
    let source = SkillSource {
        owner: "obra".to_string(),
        repo: "superpowers".to_string(),
        branch: "main".to_string(),
        folder: "skills/test-driven-development".to_string(),
    };

    let key = get_cache_key(&source);
    assert_eq!(key.len(), 64);
    assert!(key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_different_sources_different_keys() {
    let source1 = SkillSource {
        owner: "obra".to_string(),
        repo: "superpowers".to_string(),
        branch: "main".to_string(),
        folder: "skills/test-driven-development".to_string(),
    };

    let source2 = SkillSource {
        owner: "pbakaus".to_string(),
        repo: "impeccable".to_string(),
        branch: "main".to_string(),
        folder: ".agents/skills/audit".to_string(),
    };

    let key1 = get_cache_key(&source1);
    let key2 = get_cache_key(&source2);

    assert_ne!(key1, key2);
}

#[test]
fn test_same_repo_same_key() {
    let source1 = SkillSource {
        owner: "obra".to_string(),
        repo: "superpowers".to_string(),
        branch: "main".to_string(),
        folder: "skills/test-driven-development".to_string(),
    };

    let source2 = SkillSource {
        owner: "obra".to_string(),
        repo: "superpowers".to_string(),
        branch: "main".to_string(),
        folder: "skills/systematic-debugging".to_string(),
    };

    let key1 = get_cache_key(&source1);
    let key2 = get_cache_key(&source2);

    assert_eq!(key1, key2);
}
