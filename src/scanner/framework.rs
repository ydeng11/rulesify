use crate::utils::Result;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut frameworks = HashSet::new();

    if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
        if content.contains("tokio") {
            frameworks.insert("tokio");
        }
        if content.contains("actix") {
            frameworks.insert("actix");
        }
        if content.contains("serde") {
            frameworks.insert("serde");
        }
    }

    if let Ok(content) = fs::read_to_string(path.join("package.json")) {
        if content.contains("\"react\"") {
            frameworks.insert("react");
        }
        if content.contains("\"next\"") {
            frameworks.insert("nextjs");
        }
        if content.contains("\"vue\"") {
            frameworks.insert("vue");
        }
        if content.contains("\"svelte\"") {
            frameworks.insert("svelte");
        }
        if content.contains("\"express\"") {
            frameworks.insert("express");
        }
        if content.contains("\"nestjs\"") {
            frameworks.insert("nestjs");
        }
    }

    if path.join("pyproject.toml").exists() {
        if let Ok(content) = fs::read_to_string(path.join("pyproject.toml")) {
            if content.contains("django") {
                frameworks.insert("django");
            }
            if content.contains("flask") {
                frameworks.insert("flask");
            }
            if content.contains("fastapi") {
                frameworks.insert("fastapi");
            }
        }
    }

    Ok(frameworks.into_iter().map(|s| s.to_string()).collect())
}
