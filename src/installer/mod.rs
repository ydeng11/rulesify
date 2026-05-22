pub mod executor;
pub mod instructions;
pub mod tool_paths;

#[cfg(test)]
mod executor_tests;
#[cfg(test)]
mod instructions_tests;
#[cfg(test)]
mod tool_paths_tests;

pub use executor::{
    execute_npx_install, execute_npx_uninstall, install_mega_skill, install_skill,
    parse_source_url, print_install_summary, print_uninstall_summary, uninstall_skill,
    InstallResult, UninstallResult,
};
pub use instructions::{
    generate_install_instructions, generate_instructions, generate_uninstall_instructions,
    generate_uninstall_instructions_batch,
};
pub use tool_paths::{get_skill_folder, get_skill_path};

/// Given a list of tools, returns `(physical_install_tools, covered_tools)`.
///
/// When `pi` is configured alongside other agents, Pi reads skills from
/// other agent directories, so installing physically to Pi would create a
/// conflict. Pi is returned as a covered tool instead — the caller should
/// install to the other agents and record Pi as covered in the registry.
pub fn resolve_pi_coverage(tools: &[String]) -> (Vec<String>, Vec<String>) {
    if !tools.iter().any(|t| t == "pi") {
        return (tools.to_vec(), vec![]);
    }

    let other_tools: Vec<String> = tools.iter().filter(|t| *t != "pi").cloned().collect();

    if other_tools.is_empty() {
        (tools.to_vec(), vec![])
    } else {
        (other_tools, vec!["pi".to_string()])
    }
}

#[cfg(test)]
mod mod_tests {
    use super::*;

    #[test]
    fn test_no_pi_no_coverage() {
        let tools = vec!["codex".to_string(), "claude-code".to_string()];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert_eq!(physical.len(), 2);
        assert!(covered.is_empty());
    }

    #[test]
    fn test_pi_only_installs_normally() {
        let tools = vec!["pi".to_string()];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert_eq!(physical, vec!["pi".to_string()]);
        assert!(covered.is_empty());
    }

    #[test]
    fn test_pi_and_codex_skips_pi() {
        let tools = vec!["codex".to_string(), "pi".to_string()];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert_eq!(physical, vec!["codex".to_string()]);
        assert_eq!(covered, vec!["pi".to_string()]);
    }

    #[test]
    fn test_pi_and_multiple_agents_skips_pi() {
        let tools = vec![
            "codex".to_string(),
            "claude-code".to_string(),
            "cursor".to_string(),
            "pi".to_string(),
        ];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert_eq!(
            physical,
            vec![
                "codex".to_string(),
                "claude-code".to_string(),
                "cursor".to_string(),
            ]
        );
        assert_eq!(covered, vec!["pi".to_string()]);
    }

    #[test]
    fn test_pi_alone_with_other_tools_skips_pi() {
        let tools = vec!["pi".to_string(), "opencode".to_string()];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert_eq!(physical, vec!["opencode".to_string()]);
        assert_eq!(covered, vec!["pi".to_string()]);
    }

    #[test]
    fn test_empty_tools() {
        let tools: Vec<String> = vec![];
        let (physical, covered) = resolve_pi_coverage(&tools);
        assert!(physical.is_empty());
        assert!(covered.is_empty());
    }
}
