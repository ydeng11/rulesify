#[cfg(test)]
mod tests {
    use crate::models::{InstallAction, Skill};
    use crate::tui::skill_selector::{SkillSelector, SkillSelectorState};
    use std::collections::HashSet;

    fn make_skill(id: &str, name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: "Test skill".to_string(),
            source_url: "https://example.com".to_string(),
            stars: 100,
            commit_sha: "test123".to_string(),
            context_size: 1000,
            domain: "development".to_string(),
            last_updated: "2026-04-21".to_string(),
            tags: vec!["testing".to_string()],
            install_action: Some(InstallAction::Copy {
                folder: id.to_string(),
            }),
            score: Some(80.0),
            is_mega_skill: false,
            dependencies: Vec::new(),
        }
    }

    fn make_mega_skill(_id: &str, name: &str) -> Skill {
        Skill {
            name: name.to_string(),
            description: "Test mega-skill".to_string(),
            source_url: "https://example.com".to_string(),
            stars: 1000,
            commit_sha: "mega123".to_string(),
            context_size: 0,
            domain: "development".to_string(),
            last_updated: "2026-04-21".to_string(),
            tags: vec!["mega-skill".to_string()],
            install_action: Some(InstallAction::mega_skill_copy("skills", name)),
            score: Some(90.0),
            is_mega_skill: true,
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn test_installed_ids_initialized_correctly() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
            ("skill-c".to_string(), make_skill("skill-c", "Skill C")),
        ];

        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids.clone(), global_ids);

        assert_eq!(state.installed_ids, installed_ids);
        assert!(state.installed_ids.contains("skill-a"));
        assert!(state.installed_ids.contains("skill-b"));
        assert!(!state.installed_ids.contains("skill-c"));
    }

    #[test]
    fn test_global_ids_initialized_correctly() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids.clone());

        assert_eq!(state.global_ids, global_ids);
        assert!(state.global_ids.contains("skill-a"));
        assert!(!state.global_ids.contains("skill-b"));
    }

    #[test]
    fn test_selected_skill_ids_initialized_from_installed() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
            ("skill-c".to_string(), make_skill("skill-c", "Skill C")),
        ];

        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids);

        assert!(state.selected_skill_ids.contains("skill-a"));
        assert!(!state.selected_skill_ids.contains("skill-b"));
        assert!(!state.selected_skill_ids.contains("skill-c"));
    }

    #[test]
    fn test_global_ids_not_in_selected_skill_ids() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let state = SkillSelectorState::new(skills, installed_ids.clone(), global_ids);

        assert!(!state.selected_skill_ids.contains("skill-a"));
    }

    #[test]
    fn test_installed_skill_shows_i_marker() {
        let skills = vec![("skill-a".to_string(), make_skill("skill-a", "Skill A"))];

        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let _global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids, _global_ids);

        assert!(state.installed_ids.contains("skill-a"));
        assert!(!state.global_ids.contains("skill-a"));
    }

    #[test]
    fn test_global_skill_shows_g_marker() {
        let skills = vec![("skill-a".to_string(), make_skill("skill-a", "Skill A"))];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids);

        assert!(state.global_ids.contains("skill-a"));
        assert!(!state.installed_ids.contains("skill-a"));
    }

    #[test]
    fn test_global_overrides_installed_marker() {
        let skills = vec![("skill-a".to_string(), make_skill("skill-a", "Skill A"))];

        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let global_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids);

        assert!(state.global_ids.contains("skill-a"));
        assert!(state.installed_ids.contains("skill-a"));
    }

    #[test]
    fn test_uninstalled_skill_shows_empty_marker() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let _global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids, _global_ids);

        assert!(!state.installed_ids.contains("skill-a"));
        assert!(!state.global_ids.contains("skill-a"));
        assert!(!state.selected_skill_ids.contains("skill-a"));
    }

    #[test]
    fn test_selected_skill_shows_x_marker() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = HashSet::new();

        let mut state = SkillSelectorState::new(skills, installed_ids, global_ids);

        state.selected_skill_ids.insert("skill-a".to_string());

        assert!(state.selected_skill_ids.contains("skill-a"));
        assert!(!state.installed_ids.contains("skill-a"));
        assert!(!state.global_ids.contains("skill-a"));
    }

    #[test]
    fn test_installed_still_selected_not_removed() {
        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let _global_ids: HashSet<String> = HashSet::new();

        let selected_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !selected_ids.contains(*id))
            .cloned()
            .collect();

        assert!(removed.is_empty());
    }

    #[test]
    fn test_installed_unchecked_is_removed() {
        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let _global_ids: HashSet<String> = HashSet::new();

        let selected_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !selected_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&"skill-b".to_string()));
    }

    #[test]
    fn test_new_selection_is_added() {
        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let global_ids: HashSet<String> = HashSet::new();

        let selected_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();

        let added: Vec<String> = selected_ids
            .iter()
            .filter(|id| !installed_ids.contains(*id) && !global_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(added.len(), 1);
        assert!(added.contains(&"skill-b".to_string()));
    }

    #[test]
    fn test_global_skills_excluded_from_added() {
        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();

        let selected_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();

        let added: Vec<String> = selected_ids
            .iter()
            .filter(|id| !installed_ids.contains(*id) && !global_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(added.len(), 1);
        assert!(added.contains(&"skill-b".to_string()));
        assert!(!added.contains(&"skill-a".to_string()));
    }

    #[test]
    fn test_empty_selection_removes_all_installed() {
        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let _global_ids: HashSet<String> = HashSet::new();

        let selected_ids: HashSet<String> = HashSet::new();

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !selected_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(removed.len(), 2);
        assert!(removed.contains(&"skill-a".to_string()));
        assert!(removed.contains(&"skill-b".to_string()));
    }

    #[test]
    fn test_selection_result_integration() {
        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let global_ids: HashSet<String> = ["skill-c".to_string()].into_iter().collect();

        let selected_ids: HashSet<String> = [
            "skill-a".to_string(),
            "skill-c".to_string(),
            "skill-d".to_string(),
        ]
        .into_iter()
        .collect();

        let added: Vec<String> = selected_ids
            .iter()
            .filter(|id| !installed_ids.contains(*id) && !global_ids.contains(*id))
            .cloned()
            .collect();

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !selected_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(added.len(), 1);
        assert!(added.contains(&"skill-d".to_string()));

        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&"skill-b".to_string()));
    }

    #[test]
    fn test_skill_selector_empty_skills() {
        let skills: Vec<(String, Skill)> = vec![];
        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = HashSet::new();

        let selector = SkillSelector::new(skills, installed_ids, global_ids);

        let result = selector.run().unwrap();

        assert!(result.selected.is_empty());
        assert!(result.added.is_empty());
        assert!(result.removed.is_empty());
    }

    #[test]
    fn test_skill_selector_state_all_skills_filtered() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills.clone(), installed_ids, global_ids);

        assert_eq!(state.all_skills.len(), 2);
        assert_eq!(state.filtered_skills.len(), 2);
    }

    #[test]
    fn test_skill_selector_state_domains_extracted() {
        let skills = vec![
            (
                "skill-a".to_string(),
                Skill {
                    name: "Skill A".to_string(),
                    description: "Test".to_string(),
                    source_url: "https://example.com".to_string(),
                    stars: 100,
                    commit_sha: "test".to_string(),
                    context_size: 1000,
                    domain: "development".to_string(),
                    last_updated: "2026-04-21".to_string(),
                    tags: vec!["testing".to_string()],
                    install_action: None,
                    score: Some(80.0),
                    is_mega_skill: false,
                    dependencies: Vec::new(),
                },
            ),
            (
                "skill-b".to_string(),
                Skill {
                    name: "Skill B".to_string(),
                    description: "Test".to_string(),
                    source_url: "https://example.com".to_string(),
                    stars: 100,
                    commit_sha: "test".to_string(),
                    context_size: 1000,
                    domain: "data".to_string(),
                    last_updated: "2026-04-21".to_string(),
                    tags: vec!["testing".to_string()],
                    install_action: None,
                    score: Some(80.0),
                    is_mega_skill: false,
                    dependencies: Vec::new(),
                },
            ),
        ];

        let state = SkillSelectorState::new(skills, HashSet::new(), HashSet::new());

        assert!(state.domains.contains(&"All".to_string()));
        assert!(state.domains.contains(&"development".to_string()));
        assert!(state.domains.contains(&"data".to_string()));
    }

    #[test]
    fn test_skill_selector_state_tags_extracted() {
        let skills = vec![
            (
                "skill-a".to_string(),
                Skill {
                    name: "Skill A".to_string(),
                    description: "Test".to_string(),
                    source_url: "https://example.com".to_string(),
                    stars: 100,
                    commit_sha: "test".to_string(),
                    context_size: 1000,
                    domain: "development".to_string(),
                    last_updated: "2026-04-21".to_string(),
                    tags: vec!["testing".to_string(), "workflow".to_string()],
                    install_action: None,
                    score: Some(80.0),
                    is_mega_skill: false,
                    dependencies: Vec::new(),
                },
            ),
            (
                "skill-b".to_string(),
                Skill {
                    name: "Skill B".to_string(),
                    description: "Test".to_string(),
                    source_url: "https://example.com".to_string(),
                    stars: 100,
                    commit_sha: "test".to_string(),
                    context_size: 1000,
                    domain: "development".to_string(),
                    last_updated: "2026-04-21".to_string(),
                    tags: vec!["testing".to_string()],
                    install_action: None,
                    score: Some(80.0),
                    is_mega_skill: false,
                    dependencies: Vec::new(),
                },
            ),
        ];

        let state = SkillSelectorState::new(skills, HashSet::new(), HashSet::new());

        assert_eq!(state.all_tags.len(), 2);

        let testing_count = state
            .all_tags
            .iter()
            .find(|(tag, _)| tag == "testing")
            .map(|(_, count)| *count);
        assert_eq!(testing_count, Some(2));

        let workflow_count = state
            .all_tags
            .iter()
            .find(|(tag, _)| tag == "workflow")
            .map(|(_, count)| *count);
        assert_eq!(workflow_count, Some(1));
    }

    #[test]
    fn test_mega_skill_in_skills_list() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            (
                "superpowers".to_string(),
                make_mega_skill("superpowers", "Superpowers"),
            ),
        ];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids);

        let mega_skills: Vec<_> = state
            .all_skills
            .iter()
            .filter(|(_, s)| s.is_mega_skill)
            .collect();
        assert_eq!(mega_skills.len(), 1);

        let normal_skills: Vec<_> = state
            .all_skills
            .iter()
            .filter(|(_, s)| !s.is_mega_skill)
            .collect();
        assert_eq!(normal_skills.len(), 1);
    }

    #[test]
    fn test_installed_mega_skill_tagged_correctly() {
        let skills = vec![(
            "superpowers".to_string(),
            make_mega_skill("superpowers", "Superpowers"),
        )];

        let installed_ids: HashSet<String> = ["superpowers".to_string()].into_iter().collect();
        let global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills, installed_ids.clone(), global_ids);

        assert!(state.installed_ids.contains("superpowers"));
        assert!(state.selected_skill_ids.contains("superpowers"));
    }

    #[test]
    fn test_global_mega_skill_tagged_correctly() {
        let skills = vec![(
            "superpowers".to_string(),
            make_mega_skill("superpowers", "Superpowers"),
        )];

        let installed_ids: HashSet<String> = HashSet::new();
        let global_ids: HashSet<String> = ["superpowers".to_string()].into_iter().collect();

        let state = SkillSelectorState::new(skills, installed_ids, global_ids.clone());

        assert!(state.global_ids.contains("superpowers"));
    }

    #[test]
    fn test_installed_normal_skill_not_removed_when_selected() {
        let skills = vec![("skill-a".to_string(), make_skill("skill-a", "Skill A"))];

        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let _global_ids: HashSet<String> = HashSet::new();

        let state = SkillSelectorState::new(skills.clone(), installed_ids.clone(), _global_ids);

        let selected_ids = state.selected_skill_ids.clone();

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !selected_ids.contains(*id))
            .cloned()
            .collect();

        assert!(removed.is_empty());
        assert!(state.selected_skill_ids.contains("skill-a"));
    }

    #[test]
    fn test_installed_normal_skill_removed_when_unchecked() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
        ];

        let installed_ids: HashSet<String> = ["skill-a".to_string(), "skill-b".to_string()]
            .into_iter()
            .collect();
        let global_ids: HashSet<String> = HashSet::new();

        let mut state = SkillSelectorState::new(skills, installed_ids.clone(), global_ids);

        state.selected_skill_ids.remove("skill-b");

        let removed: Vec<String> = installed_ids
            .iter()
            .filter(|id| !state.selected_skill_ids.contains(*id))
            .cloned()
            .collect();

        assert_eq!(removed.len(), 1);
        assert!(removed.contains(&"skill-b".to_string()));
    }

    #[test]
    fn test_globally_installed_skills_marked_when_running_tool() {
        let skills = vec![
            ("skill-a".to_string(), make_skill("skill-a", "Skill A")),
            ("skill-b".to_string(), make_skill("skill-b", "Skill B")),
            (
                "superpowers".to_string(),
                make_mega_skill("superpowers", "Superpowers"),
            ),
        ];

        let installed_ids: HashSet<String> = ["skill-a".to_string()].into_iter().collect();
        let global_ids: HashSet<String> = ["skill-b".to_string(), "superpowers".to_string()]
            .into_iter()
            .collect();

        let state = SkillSelectorState::new(skills, installed_ids.clone(), global_ids.clone());

        assert!(state.global_ids.contains("skill-b"));
        assert!(state.global_ids.contains("superpowers"));

        assert!(!state.selected_skill_ids.contains("skill-b"));
        assert!(!state.selected_skill_ids.contains("superpowers"));

        assert!(state.installed_ids.contains("skill-a"));
        assert!(state.selected_skill_ids.contains("skill-a"));

        assert!(!state.global_ids.contains("skill-a"));
    }
}
