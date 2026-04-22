use crate::utils::Result;
use std::collections::HashSet;
use std::path::Path;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut tools = HashSet::new();

    if path.join(".cursor").exists() || path.join(".cursorrules").exists() {
        tools.insert("cursor");
    }

    if path.join("CLAUDE.md").exists() || path.join(".claude").exists() {
        tools.insert("claude-code");
    }

    if path.join(".agents").exists() || path.join("AGENTS.md").exists() {
        tools.insert("codex");
    }

    if path.join(".opencode").exists() {
        tools.insert("opencode");
    }

    if path.join(".pi").exists() {
        tools.insert("pi");
    }

    Ok(tools.into_iter().map(|s| s.to_string()).collect())
}
