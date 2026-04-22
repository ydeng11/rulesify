#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, RepoMetrics, SkillMetadata};
    use crate::registry::Scorer;
    use chrono::{Duration as ChronoDuration, Utc};

    fn make_meta() -> SkillMetadata {
        SkillMetadata {
            skill_id: "test".into(),
            name: "Test".into(),
            description: "Test skill description for testing".into(),
            source_repo: "anthropics/skills".into(),
            source_folder: "test/SKILL.md".into(),
            source_url: "https://github.com/anthropics/skills/tree/main/test".into(),
            commit_sha: "abc123".into(),
            tags: vec!["test".into()],
            stars: 5000,
            context_size: 1000,
            domain: "development".into(),
            last_updated: "2026-04-10".into(),
            install_action: InstallAction::Copy {
                folder: "test".into(),
            },
            is_mega_skill: false,
            dependencies: Vec::new(),
        }
    }

    fn make_mega_skill_meta() -> SkillMetadata {
        SkillMetadata {
            skill_id: "superpowers".into(),
            name: "superpowers".into(),
            description: "Complete software development methodology for coding agents".into(),
            source_repo: "obra/superpowers".into(),
            source_folder: "skills".into(),
            source_url: "https://github.com/obra/superpowers/tree/main/skills".into(),
            commit_sha: String::new(),
            tags: vec!["mega-skill".into()],
            stars: 160000,
            context_size: 0,
            domain: "development".into(),
            last_updated: "2026-04-20".into(),
            install_action: InstallAction::mega_skill_copy("skills", "superpowers"),
            is_mega_skill: true,
            dependencies: Vec::new(),
        }
    }

    fn make_metrics(stars: u32, days_since_push: i64) -> RepoMetrics {
        RepoMetrics {
            stars,
            pushed_at: Utc::now() - ChronoDuration::days(days_since_push),
            issues_open_3mo: 5,
            issues_closed_3mo: 10,
            contributors_total: 20,
            contributors_active_3mo: 8,
        }
    }

    #[test]
    fn test_score_calculation() {
        let scorer = Scorer::default();
        let metrics = make_metrics(5000, 10);
        let score = scorer.calculate(&metrics);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_mega_skill_score_bonus() {
        let scorer = Scorer::default();
        let metrics = make_metrics(5000, 10);

        let regular_meta = make_meta();
        let regular_score = scorer.calculate_for_skill(&regular_meta, &metrics);

        let mega_meta = make_mega_skill_meta();
        let mega_score = scorer.calculate_for_mega_skill(&mega_meta, &metrics);

        assert!(mega_score > regular_score);
        let diff = mega_score - regular_score;
        assert!(diff >= 20.0);
    }

    #[test]
    fn test_stars_score() {
        let scorer = Scorer::default();

        let metrics_low = make_metrics(1000, 0);
        let score_low = scorer.calculate(&metrics_low);

        let metrics_high = make_metrics(10000, 0);
        let score_high = scorer.calculate(&metrics_high);

        assert!(score_high > score_low);
    }

    #[test]
    fn test_recency_score() {
        let scorer = Scorer::default();

        let recent = make_metrics(5000, 10);
        let medium = make_metrics(5000, 45);
        let old = make_metrics(5000, 120);

        let score_recent = scorer.calculate(&recent);
        let score_medium = scorer.calculate(&medium);
        let score_old = scorer.calculate(&old);

        assert!(score_recent > score_medium);
        assert!(score_medium > score_old);
    }

    #[test]
    fn test_issue_resolution_score() {
        let scorer = Scorer::default();

        let good_resolution = RepoMetrics {
            stars: 5000,
            pushed_at: Utc::now(),
            issues_open_3mo: 2,
            issues_closed_3mo: 18,
            contributors_total: 10,
            contributors_active_3mo: 5,
        };

        let poor_resolution = RepoMetrics {
            stars: 5000,
            pushed_at: Utc::now(),
            issues_open_3mo: 15,
            issues_closed_3mo: 5,
            contributors_total: 10,
            contributors_active_3mo: 5,
        };

        let score_good = scorer.calculate(&good_resolution);
        let score_poor = scorer.calculate(&poor_resolution);

        assert!(score_good > score_poor);
    }

    #[test]
    fn test_high_score() {
        let scorer = Scorer::default();
        let metrics = RepoMetrics {
            stars: 10000,
            pushed_at: Utc::now() - ChronoDuration::days(5),
            issues_open_3mo: 2,
            issues_closed_3mo: 18,
            contributors_total: 50,
            contributors_active_3mo: 40,
        };
        let score = scorer.calculate(&metrics);
        assert!(score > 80.0);
    }

    #[test]
    fn test_filter_and_sort() {
        let scorer = Scorer::default();
        let meta = make_meta();
        let skills = vec![
            (meta.clone(), 60.0),
            (meta.clone(), 75.0),
            (meta.clone(), 90.0),
            (meta.clone(), 50.0),
        ];

        let filtered = scorer.filter_above_threshold(skills, 60.0);
        assert_eq!(filtered.len(), 3);

        let sorted = scorer.sort_and_limit(filtered, 2);
        assert_eq!(sorted.len(), 2);
        assert!(sorted[0].1 >= sorted[1].1);
    }

    #[test]
    fn test_mega_skill_with_npx() {
        let scorer = Scorer::default();
        let metrics = make_metrics(200, 5);

        let gsd_meta = SkillMetadata {
            skill_id: "gsd".into(),
            name: "gsd".into(),
            description: "Get Shit Done project management system".into(),
            source_repo: "gsd-build/get-shit-done".into(),
            source_folder: String::new(),
            source_url: "https://github.com/gsd-build/get-shit-done".into(),
            commit_sha: String::new(),
            tags: vec!["mega-skill".into()],
            stars: 200,
            context_size: 0,
            domain: "development".into(),
            last_updated: "2026-04-20".into(),
            install_action: InstallAction::Npx {
                package: "get-shit-done-cc".into(),
                args: vec!["@latest".into()],
                uninstall_flag: None,
            },
            is_mega_skill: true,
            dependencies: Vec::new(),
        };

        let score = scorer.calculate_for_mega_skill(&gsd_meta, &metrics);
        assert!(score > 20.0);
    }
}
