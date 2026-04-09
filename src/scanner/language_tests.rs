#[cfg(test)]
mod tests {
    use crate::scanner::language::detect;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        
        let langs = detect(dir.path()).unwrap();
        assert!(langs.contains(&"rust".to_string()));
    }

    #[test]
    fn test_detect_typescript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("app.ts"), "").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        
        let langs = detect(dir.path()).unwrap();
        assert!(langs.contains(&"typescript".to_string()));
    }
    
    #[test]
    fn test_detect_multiple_languages() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::write(dir.path().join("app.ts"), "").unwrap();
        fs::write(dir.path().join("lib.py"), "").unwrap();
        
        let langs = detect(dir.path()).unwrap();
        assert!(langs.contains(&"rust".to_string()));
        assert!(langs.contains(&"typescript".to_string()));
        assert!(langs.contains(&"python".to_string()));
    }
}