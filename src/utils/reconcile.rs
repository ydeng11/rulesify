use crate::installer::tool_paths::get_skill_folder;
use crate::models::{GlobalConfig, ProjectConfig, Scope};

pub fn skill_exists_on_disk(tool: &str, scope: Scope, skill_id: &str) -> bool {
    let skill_folder = get_skill_folder(tool, scope, skill_id);
    skill_folder.exists() && skill_folder.join("SKILL.md").exists()
}

pub fn reconcile_global_config(config: &mut GlobalConfig) -> Vec<(String, String)> {
    let mut removed = Vec::new();

    let tools_to_check: Vec<String> = config.installed_skills.keys().cloned().collect();

    for tool in tools_to_check {
        if let Some(skills) = config.installed_skills.get_mut(&tool) {
            let stale_skills: Vec<String> = skills
                .iter()
                .filter(|(id, _)| !skill_exists_on_disk(&tool, Scope::Global, id))
                .map(|(id, _)| id.clone())
                .collect();

            for id in stale_skills {
                skills.remove(&id);
                removed.push((tool.clone(), id));
            }
        }
    }

    config
        .installed_skills
        .retain(|_, skills| !skills.is_empty());

    removed
}

pub fn reconcile_project_config(config: &mut ProjectConfig) -> Vec<String> {
    let mut removed = Vec::new();

    let stale_skills: Vec<String> = config
        .installed_skills
        .iter()
        .filter(|(id, _)| {
            config
                .tools
                .iter()
                .all(|tool| !skill_exists_on_disk(tool, Scope::Project, id))
        })
        .map(|(id, _)| id.clone())
        .collect();

    for id in &stale_skills {
        config.installed_skills.remove(id);
        removed.push(id.clone());
    }

    removed
}
