use crate::utils::Result;
use std::collections::HashSet;
use std::path::Path;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut tools = HashSet::new();

    if path.join(".cursor").exists() || path.join(".cursorrules").exists() {
        tools.insert("cursor");
    }

    if path.join("CLAUDE.md").exists() {
        tools.insert("claude-code");
    }

    if path.join(".clinerules").exists() {
        tools.insert("cline");
    }

    if path.join(".goosehints").exists() {
        tools.insert("goose");
    }

    Ok(tools.into_iter().map(|s| s.to_string()).collect())
}
