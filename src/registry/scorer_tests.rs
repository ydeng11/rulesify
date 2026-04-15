#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, SkillMetadata};
    use crate::registry::Scorer;

    fn make_meta(stars: u32) -> SkillMetadata {
        SkillMetadata {
            skill_id: "test".into(),
            name: "Test".into(),
            description: "Test skill description for testing".into(),
            source_repo: "anthropics/skills".into(),
            source_path: "test/SKILL.md".into(),
            source_url: "https://github.com/anthropics/skills/tree/main/test".into(),
            tags: vec!["test".into()],
            stars,
            context_size: 1000,
            domain: "development".into(),
            last_updated: "2026-04-10".into(),
            install_action: InstallAction::Copy {
                folder: "test".into(),
            },
        }
    }

    #[test]
    fn test_score_calculation() {
        let scorer = Scorer::default();
        let meta = make_meta(5000);
        let score = scorer.calculate(&meta);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_high_score() {
        let scorer = Scorer::default();
        let meta = make_meta(10000);
        let score = scorer.calculate(&meta);
        assert!(score > 80.0);
    }

    #[test]
    fn test_filter_and_sort() {
        let scorer = Scorer::default();
        let skills = vec![
            (make_meta(1000), 60.0),
            (make_meta(5000), 75.0),
            (make_meta(10000), 90.0),
            (make_meta(500), 50.0),
        ];

        let filtered = scorer.filter_above_threshold(skills, 60.0);
        assert_eq!(filtered.len(), 3);

        let sorted = scorer.sort_and_limit(filtered, 2);
        assert_eq!(sorted.len(), 2);
        assert!(sorted[0].1 >= sorted[1].1);
    }
}
