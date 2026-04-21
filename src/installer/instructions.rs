use crate::installer::tool_paths::{get_skill_folder, get_skill_path};
use crate::models::{InstallAction, Scope, Skill};

pub fn generate_instructions(skills: &[(String, Skill)], tools: &[String]) -> String {
    let mut output = String::new();

    output.push_str("# Installation Instructions\n\n");
    output.push_str("Copy the following instructions for your AI agent:\n\n");
    output.push_str("---\n\n");

    for (_id, skill) in skills {
        output.push_str(&format!("## Skill: {}\n\n", skill.name));
        output.push_str(&format!("**Source:** {}\n\n", skill.source_url));

        if skill.is_mega_skill {
            output.push_str("**Type:** Mega-skill (collection of sub-skills)\n\n");
        }

        for tool in tools {
            output.push_str(&format!("### For {}\n\n", tool));
            output.push_str(&generate_install_for_skill(tool, skill));
        }

        output.push_str("\n---\n\n");
    }

    output
}

fn generate_install_for_skill(tool: &str, skill: &Skill) -> String {
    if let Some(action) = &skill.install_action {
        match action {
            InstallAction::MegaSkillCopy {
                source_folder,
                dest_name,
            } => generate_mega_skill_copy_instructions(source_folder, dest_name, tool, skill),
            InstallAction::Npx {
                package,
                args,
                uninstall_flag: _,
            } => generate_npx_instructions(package, args),
            InstallAction::Copy { folder } => generate_copy_instructions(folder, tool, &skill.name),
            InstallAction::Command { value } => generate_command_instructions(value),
        }
    } else {
        generate_copy_instructions("", tool, &skill.name)
    }
}

fn generate_mega_skill_copy_instructions(
    source_folder: &str,
    dest_name: &str,
    tool: &str,
    skill: &Skill,
) -> String {
    let repo_url = skill
        .source_url
        .replace("/tree/main/", "/archive/refs/heads/main.zip");

    let dest_folder = format!("{}/skills/{}", get_tool_base_path(tool), dest_name);

    format!(
        "This is a mega-skill. Install by copying the entire `{}` folder:\n\n\
         1. Download from GitHub:\n   {}\n\n\
         2. Extract the archive\n\n\
         3. Copy the `{}` folder to:\n   {}\n\n\
         4. Rename the copied folder to `{}`\n\n\
         Result: The destination should contain all sub-skills as subdirectories.\n\n",
        source_folder, repo_url, source_folder, dest_folder, dest_name
    )
}

fn generate_npx_instructions(package: &str, args: &[String]) -> String {
    let full_command = if args.is_empty() {
        format!("npx {}", package)
    } else {
        format!("npx {} {}", package, args.join(" "))
    };

    format!(
        "Install using npx:\n\n\
         ```bash\n{}\n```\n\n\
         This will automatically install the skill to the correct location.\n\n",
        full_command
    )
}

fn generate_copy_instructions(folder: &str, tool: &str, skill_name: &str) -> String {
    let skill_path = get_skill_path(tool, Scope::Project, skill_name);
    let skill_folder = get_skill_folder(tool, Scope::Project, skill_name);

    if folder.is_empty() {
        format!(
            "1. Create folder:\n   {}\n\n\
             2. Create SKILL.md file:\n   {}\n\n",
            skill_folder.display(),
            skill_path.display()
        )
    } else {
        format!(
            "1. Copy folder `{}` to:\n   {}\n\n\
             2. Ensure SKILL.md exists at:\n   {}\n\n",
            folder,
            skill_folder.display(),
            skill_path.display()
        )
    }
}

fn generate_command_instructions(value: &str) -> String {
    format!("Run the following command:\n\n```bash\n{}\n```\n\n", value)
}

fn get_tool_base_path(tool: &str) -> &'static str {
    match tool {
        "claude-code" => ".claude",
        "codex" => ".agents",
        "cursor" => ".cursor",
        "opencode" => ".opencode",
        "pi" => ".pi/skills/pi-skills",
        _ => ".agents",
    }
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
