#[cfg(test)]
mod tests {
    use crate::registry::SkillParser;

    #[test]
    fn test_parse_valid() {
        let content = "---\nname: tdd\ndescription: Test driven development methodology\ntags: [testing]\n---\n\n# TDD\n\nContent...";
        let parsed = SkillParser::parse(content).unwrap();
        assert_eq!(parsed.name, "tdd");
        assert!(parsed.description.len() >= 20);
        assert!(!parsed.is_mega_skill);
    }

    #[test]
    fn test_parse_mega_skill() {
        let content = "---\nname: superpowers\ndescription: Complete software development methodology for coding agents - brainstorming, TDD, systematic debugging\nis_mega_skill: true\ntags: [workflow, testing, debugging]\n---\n\n# Superpowers\n\nMega-skill collection...";
        let parsed = SkillParser::parse(content).unwrap();
        assert_eq!(parsed.name, "superpowers");
        assert!(parsed.is_mega_skill);
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# No frontmatter";
        assert!(SkillParser::parse(content).is_err());
    }

    #[test]
    fn test_parse_short_description() {
        let content = "---\nname: test\ndescription: too short\n---\n\n# Test";
        assert!(SkillParser::parse(content).is_err());
    }

    #[test]
    fn test_estimate_context_size() {
        let content = "---\nname: test\ndescription: A long enough description here\n---\n\n# Test\n\nSome content\nMore lines\nEven more";
        let size = SkillParser::estimate_context_size(content);
        assert!(size > 0);
    }

    #[test]
    fn test_parse_is_mega_skill_defaults_false() {
        let content = "---\nname: test-skill\ndescription: A regular skill without mega-skill flag\n---\n\n# Test\n\nRegular skill content...";
        let parsed = SkillParser::parse(content).unwrap();
        assert!(!parsed.is_mega_skill);
    }
}
