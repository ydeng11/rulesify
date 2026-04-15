#[cfg(test)]
mod tests {
    use crate::registry::SourceRepo;

    #[test]
    fn test_all_sources() {
        let sources = SourceRepo::all();
        assert!(sources.len() >= 5);
    }

    #[test]
    fn test_source_properties() {
        let anthropic = SourceRepo::AnthropicSkills;
        assert_eq!(anthropic.owner(), "anthropics");
        assert_eq!(anthropic.repo(), "skills");
        assert_eq!(anthropic.branch(), "main");
        assert_eq!(anthropic.skill_pattern(), "skills/*/SKILL.md");
    }

    #[test]
    fn test_parse_skill_id() {
        let anthropic = SourceRepo::AnthropicSkills;
        let id = anthropic.parse_skill_id("skills/tdd/SKILL.md");
        assert_eq!(id, Some("tdd".to_string()));

        let mattpocock = SourceRepo::MattPocockSkills;
        let id = mattpocock.parse_skill_id("brainstorming/SKILL.md");
        assert_eq!(id, Some("brainstorming".to_string()));

        let openai = SourceRepo::OpenAISkillsCurated;
        let id = openai.parse_skill_id("skills/.curated/gh-fix-ci/SKILL.md");
        assert_eq!(id, Some("gh-fix-ci".to_string()));
    }

    #[test]
    fn test_parse_skill_folder() {
        let anthropic = SourceRepo::AnthropicSkills;
        let folder = anthropic.parse_skill_folder("skills/tdd/SKILL.md");
        assert_eq!(folder, Some("skills/tdd".to_string()));

        let mattpocock = SourceRepo::MattPocockSkills;
        let folder = mattpocock.parse_skill_folder("brainstorming/SKILL.md");
        assert_eq!(folder, Some("brainstorming".to_string()));

        let no_folder = anthropic.parse_skill_folder("skills/tdd/README.md");
        assert_eq!(no_folder, None);
    }
}
