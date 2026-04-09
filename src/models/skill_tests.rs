#[cfg(test)]
mod tests {
    use crate::models::Skill;

    #[test]
    fn test_matches_tools() {
        let skill = Skill {
            name: "Test".into(),
            description: "Desc".into(),
            source: "url".into(),
            tags: vec!["testing".into()],
            compatible_tools: vec!["cursor".into(), "claude-code".into()],
            popularity: 100,
        };
        
        assert!(skill.matches_tools(&["cursor".to_string()]));
        assert!(skill.matches_tools(&["claude-code".to_string()]));
        assert!(!skill.matches_tools(&["cline".to_string()]));
    }

    #[test]
    fn test_matches_tags() {
        let skill = Skill {
            name: "Test".into(),
            description: "Desc".into(),
            source: "url".into(),
            tags: vec!["testing".into(), "rust".into()],
            compatible_tools: vec!["cursor".into()],
            popularity: 100,
        };
        
        assert!(skill.matches_tags(&["testing".to_string()]));
        assert!(skill.matches_tags(&["rust".to_string()]));
        assert!(!skill.matches_tags(&["python".to_string()]));
    }
    
    #[test]
    fn test_matches_multiple_tags() {
        let skill = Skill {
            name: "Multi".into(),
            description: "Multi-tag skill".into(),
            source: "url".into(),
            tags: vec!["rust".into(), "testing".into(), "debugging".into()],
            compatible_tools: vec!["cursor".into()],
            popularity: 100,
        };
        
        assert!(skill.matches_tags(&["python".to_string(), "rust".to_string()]));
    }
}