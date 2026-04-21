#[cfg(test)]
mod tests {
    use crate::registry::SourceRepo;

    #[test]
    fn test_all_sources() {
        let sources = SourceRepo::all();
        assert!(sources.len() >= 10);
    }

    #[test]
    fn test_source_properties() {
        let anthropic = SourceRepo::AnthropicSkills;
        assert_eq!(anthropic.owner(), "anthropics");
        assert_eq!(anthropic.repo(), "skills");
        assert_eq!(anthropic.branch(), "main");
        assert_eq!(anthropic.skill_pattern(), "skills/*/SKILL.md");
        assert!(!anthropic.is_mega_skill_collection());
    }

    #[test]
    fn test_mega_skill_sources() {
        let superpowers = SourceRepo::ObraSuperpowers;
        assert!(superpowers.is_mega_skill_collection());
        assert_eq!(superpowers.owner(), "obra");
        assert_eq!(superpowers.repo(), "superpowers");
        assert_eq!(superpowers.skill_pattern(), "");
        assert_eq!(superpowers.mega_skill_source_folder(), "skills");
        assert_eq!(superpowers.mega_skill_dest_name(), "superpowers");

        let gsd = SourceRepo::GsdSkills;
        assert!(gsd.is_mega_skill_collection());
        assert_eq!(gsd.owner(), "gsd-build");
        assert_eq!(gsd.repo(), "get-shit-done");
        assert_eq!(gsd.mega_skill_dest_name(), "gsd");

        let impeccable = SourceRepo::PbakausImpeccable;
        assert!(impeccable.is_mega_skill_collection());
        assert_eq!(impeccable.owner(), "pbakaus");
        assert_eq!(impeccable.repo(), "impeccable");
        assert_eq!(
            impeccable.skill_pattern(),
            "source/skills/impeccable/SKILL.md"
        );
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

        let impeccable = SourceRepo::PbakausImpeccable;
        let id = impeccable.parse_skill_id("source/skills/impeccable/SKILL.md");
        assert_eq!(id, Some("impeccable".to_string()));
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

    #[test]
    fn test_mega_skill_sources_parse_none() {
        let superpowers = SourceRepo::ObraSuperpowers;
        assert_eq!(
            superpowers.parse_skill_id("skills/brainstorming/SKILL.md"),
            None
        );

        let gsd = SourceRepo::GsdSkills;
        assert_eq!(gsd.parse_skill_id("commands/debug/SKILL.md"), None);
    }
}
