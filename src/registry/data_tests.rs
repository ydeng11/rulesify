#[cfg(test)]
mod tests {
    use crate::models::InstallAction;
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
        assert!(registry.get_skill("superpowers").is_some());
    }

    #[test]
    fn curated_skills_install_from_pinned_upstream_folders() {
        let registry = load_builtin().unwrap();

        for (id, repo, folder) in [
            (
                "diagram-design",
                "cathrynlavery/diagram-design",
                "skills/diagram-design",
            ),
            (
                "vercel-react-best-practices",
                "vercel-labs/agent-skills",
                "skills/react-best-practices",
            ),
            ("xlsx", "anthropics/skills", "skills/xlsx"),
            (
                "agent-browser",
                "vercel-labs/agent-browser",
                "skills/agent-browser",
            ),
        ] {
            let skill = registry
                .get_skill(id)
                .unwrap_or_else(|| panic!("Missing {id}"));
            assert_eq!(skill.name, id);
            assert_eq!(
                skill.source_url,
                format!("https://github.com/{repo}/tree/main/{folder}")
            );
            assert!(
                matches!(&skill.install_action, Some(InstallAction::Copy { folder: source }) if source == folder)
            );
            assert_eq!(skill.commit_sha.len(), 40, "{id} must pin a commit");
            assert!(skill
                .commit_sha
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit()));
        }
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
