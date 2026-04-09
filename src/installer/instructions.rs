use crate::models::Skill;

pub fn generate_instructions(skills: &[(String, Skill)], tools: &[String]) -> String {
    let mut output = String::new();
    
    output.push_str("# Installation Instructions\n\n");
    output.push_str("Copy the following instructions for your AI agent:\n\n");
    output.push_str("---\n\n");
    
    for (_id, skill) in skills {
        output.push_str(&format!("## Skill: {}\n\n", skill.name));
        output.push_str(&format!("**Source:** {}\n\n", skill.source));
        
        for tool in tools {
            if skill.compatible_tools.contains(tool) {
                output.push_str(&format!("### For {}\n\n", tool));
                output.push_str(&generate_tool_specific_instructions(tool, &skill.source));
            }
        }
        
        output.push_str("\n---\n\n");
    }
    
    output
}

fn generate_tool_specific_instructions(tool: &str, source: &str) -> String {
    match tool {
        "cursor" => format!(
            "1. Fetch the skill instructions from: {}\n2. Create `.cursor/rules/<skill-name>.md` with the content\n",
            source
        ),
        "claude-code" => format!(
            "1. Fetch the skill instructions from: {}\n2. Append to `CLAUDE.md` or create a dedicated section\n",
            source
        ),
        "cline" => format!(
            "1. Fetch the skill instructions from: {}\n2. Add to `.clinerules` file\n",
            source
        ),
        "goose" => format!(
            "1. Fetch the skill instructions from: {}\n2. Add to `.goosehints` file\n",
            source
        ),
        _ => format!("Install from: {}\n", source),
    }
}