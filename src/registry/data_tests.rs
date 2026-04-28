#[cfg(test)]
mod tests {
    use crate::registry::load_builtin;

    #[test]
    fn test_load_builtin() {
        let registry = load_builtin().unwrap();
        assert!(!registry.skills.is_empty());
        assert_eq!(registry.version, 1);
    }

    #[test]
    fn test_skill_exists() {
        let registry = load_builtin().unwrap();
        assert!(registry.get_skill("tdd").is_some());
    }

    #[test]
    fn test_filter_by_domain() {
        let registry = load_builtin().unwrap();
        let filtered = registry.filter_by_domain("development");
        assert!(!filtered.is_empty());

        for (_, skill) in filtered {
            assert_eq!(skill.domain, "development");
        }
    }
}
