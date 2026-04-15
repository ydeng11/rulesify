use crate::utils::Result;
use std::collections::HashSet;
use std::path::Path;
use walkdir::WalkDir;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut languages = HashSet::new();

    for entry in WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match ext {
            "rs" => {
                languages.insert("rust");
            }
            "ts" | "tsx" => {
                languages.insert("typescript");
            }
            "js" | "jsx" => {
                languages.insert("javascript");
            }
            "py" => {
                languages.insert("python");
            }
            "go" => {
                languages.insert("go");
            }
            "java" => {
                languages.insert("java");
            }
            "rb" => {
                languages.insert("ruby");
            }
            "php" => {
                languages.insert("php");
            }
            "c" | "cpp" | "cc" => {
                languages.insert("cpp");
            }
            _ => {}
        }
    }

    if path.join("Cargo.toml").exists() {
        languages.insert("rust");
    }
    if path.join("package.json").exists() {
        if !languages.contains("typescript") {
            languages.insert("javascript");
        }
    }
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        languages.insert("python");
    }
    if path.join("go.mod").exists() {
        languages.insert("go");
    }

    Ok(languages.into_iter().map(|s| s.to_string()).collect())
}
