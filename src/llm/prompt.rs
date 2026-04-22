use crate::models::Domain;

pub fn build_prompts() -> (String, String) {
    let system_prompt = build_system_prompt();
    let empty_user_prompt = String::new();
    (system_prompt, empty_user_prompt)
}

fn build_system_prompt() -> String {
    format!(
        r#"You classify AI agent skills into domains and assign relevant tags.

Domains (choose one):
{}

Tags: Choose up to 3 tags describing the skill's capabilities. Tags should be lowercase, hyphenated if needed, and specific to the skill's purpose.

Input format: {{ "<skill>": {{ "description": "<words>" }} }}
Output format: {{ "<skill>": {{ "domain": "<domain>", "tags": ["<tag1>", "<tag2>"] }} }}

Classify each skill and respond ONLY with the JSON output format. Do not include any explanation or additional text."#,
        Domain::domain_list_string()
    )
}

pub fn build_user_prompt(skills: &[(String, String)]) -> String {
    let mut skills_map = serde_json::Map::new();
    for (skill_id, description) in skills {
        skills_map.insert(
            skill_id.clone(),
            serde_json::json!({ "description": description }),
        );
    }
    serde_json::to_string(&serde_json::Value::Object(skills_map)).unwrap_or_default()
}
