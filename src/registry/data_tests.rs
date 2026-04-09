#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_builtin() {
        let registry = load_builtin().unwrap();
        assert!(registry.skills.len() > 0);
        assert_eq!(registry.version, 1);
    }

    #[test]
    fn test_skill_exists() {
        let registry = load_builtin().unwrap();
        assert!(registry.get_skill("test-driven-development").is_some());
    }
    
    #[test]
    fn test_filter_by_tools() {
        let registry = load_builtin().unwrap();
        let filtered = registry.filter_by_tools(&["cursor".to_string()]);
        assert!(filtered.len() > 0);
        
        for (_, skill) in filtered {
            assert!(skill.compatible_tools.contains(&"cursor".to_string()));
        }
    }
}