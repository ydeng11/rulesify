use crate::installer::tool_paths::{get_skill_folder, get_skill_path};
use crate::models::{Scope, Skill};

pub fn generate_instructions(skills: &[(String, Skill)], tools: &[String]) -> String {
    let mut output = String::new();

    output.push_str("# Installation Instructions\n\n");
    output.push_str("Copy the following instructions for your AI agent:\n\n");
    output.push_str("---\n\n");

    for (_id, skill) in skills {
        output.push_str(&format!("## Skill: {}\n\n", skill.name));
        output.push_str(&format!("**Source:** {}\n\n", skill.source_url));

        for tool in tools {
            output.push_str(&format!("### For {}\n\n", tool));
            output.push_str(&generate_tool_specific_instructions(
                tool,
                &skill.source_url,
                &skill.name,
            ));
        }

        output.push_str("\n---\n\n");
    }

    output
}

pub fn generate_install_instructions(
    skill_name: &str,
    source_url: &str,
    tools: &[String],
    scope: Scope,
) -> String {
    let mut output = String::new();
    let scope_label = match scope {
        Scope::Project => "project",
        Scope::Global => "global",
    };

    output.push_str("# Install Instructions\n\n");
    output.push_str(&format!("Scope: {} level\n\n", scope_label));

    for tool in tools {
        let skill_path = get_skill_path(tool, scope.clone(), skill_name);
        let skill_folder = get_skill_folder(tool, scope.clone(), skill_name);

        output.push_str(&format!("## {} ({})\n\n", tool, scope_label));
        output.push_str(&format!("1. Fetch SKILL.md from:\n   {}\n\n", source_url));
        output.push_str(&format!(
            "2. Create folder:\n   {}\n\n",
            skill_folder.display()
        ));
        output.push_str(&format!(
            "3. Copy SKILL.md to:\n   {}\n\n",
            skill_path.display()
        ));
        output.push_str("---\n\n");
    }

    output
}

pub fn generate_uninstall_instructions(skill_name: &str, tools: &[String], scope: Scope) -> String {
    let mut output = String::new();
    let scope_label = match scope {
        Scope::Project => "project",
        Scope::Global => "global",
    };

    output.push_str("# Uninstall Instructions\n\n");
    output.push_str(&format!("Scope: {} level\n\n", scope_label));

    for tool in tools {
        let skill_folder = get_skill_folder(tool, scope.clone(), skill_name);

        output.push_str(&format!("## {} ({})\n\n", tool, scope_label));
        output.push_str(&format!(
            "Delete folder:\n   {}\n\n",
            skill_folder.display()
        ));
        output.push_str("---\n\n");
    }

    output
}

pub fn generate_uninstall_instructions_batch(
    skill_ids: &[String],
    tools: &[String],
    scope: Scope,
) -> String {
    let mut output = String::new();
    let scope_label = match scope {
        Scope::Project => "project",
        Scope::Global => "global",
    };

    output.push_str("# Uninstall Instructions\n\n");
    output.push_str(&format!("Scope: {} level\n\n", scope_label));

    for skill_id in skill_ids {
        output.push_str(&format!("## Skill: {}\n\n", skill_id));

        for tool in tools {
            let skill_folder = get_skill_folder(tool, scope.clone(), skill_id);

            output.push_str(&format!("### {} ({})\n\n", tool, scope_label));
            output.push_str(&format!(
                "Delete folder:\n   {}\n\n",
                skill_folder.display()
            ));
        }

        output.push_str("---\n\n");
    }

    output
}

fn generate_tool_specific_instructions(tool: &str, source: &str, skill_name: &str) -> String {
    match tool {
        "cursor" => format!(
            "1. Fetch SKILL.md from: {}\n2. Create `.cursor/skills/{}/SKILL.md`\n",
            source, skill_name
        ),
        "claude-code" => format!(
            "1. Fetch SKILL.md from: {}\n2. Create `.claude/skills/{}/SKILL.md`\n",
            source, skill_name
        ),
        "codex" => format!(
            "1. Fetch SKILL.md from: {}\n2. Create `.agents/skills/{}/SKILL.md`\n",
            source, skill_name
        ),
        "opencode" => format!(
            "1. Fetch SKILL.md from: {}\n2. Create `.opencode/skills/{}/SKILL.md`\n",
            source, skill_name
        ),
        "pi" => format!(
            "1. Fetch SKILL.md from: {}\n2. Create `.pi/skills/pi-skills/{}/SKILL.md`\n",
            source, skill_name
        ),
        _ => format!("Install from: {}\n", source),
    }
}
