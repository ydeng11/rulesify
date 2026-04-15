use anyhow::Result;
use rulesify::{
    models::{InstallAction, SkillMetadata},
    registry::{GitHubClient, RegistryGenerator, Scorer, SkillParser, SourceRepo},
};
use std::collections::HashMap;

async fn fetch_skill(
    client: &GitHubClient,
    source: SourceRepo,
    path: &str,
    repo_stars: u32,
) -> Result<SkillMetadata> {
    let content = client
        .fetch_file(source.owner(), source.repo(), path)
        .await?;
    let parsed = SkillParser::parse(&content)?;
    let skill_id = source.parse_skill_id(path).unwrap_or("unknown".into());
    let context_size = SkillParser::estimate_context_size(&content);

    let source_url = format!(
        "https://github.com/{}/skills/tree/{}/{}",
        source.owner(),
        source.branch(),
        path.replace("/SKILL.md", "")
    );

    Ok(SkillMetadata {
        skill_id,
        name: parsed.name,
        description: parsed.description,
        source_repo: source.full_name(),
        source_path: path.into(),
        source_url,
        tags: parsed.tags,
        stars: repo_stars,
        context_size,
        domain: source.domain().into(),
        last_updated: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        install_action: InstallAction::Copy {
            folder: path.replace("/SKILL.md", ""),
        },
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let token = std::env::var("GITHUB_TOKEN").ok();
    let client = GitHubClient::new(token);
    let scorer = Scorer::default();

    let mut all_skills: Vec<(SkillMetadata, f32)> = vec![];

    for source in SourceRepo::all() {
        log::info!("Fetching from {}", source.full_name());

        let repo = match client.fetch_repo(source.owner(), source.repo()).await {
            Ok(r) => r,
            Err(e) => {
                log::warn!("Failed to fetch repo {}: {}", source.full_name(), e);
                continue;
            }
        };

        let tree = match client
            .fetch_tree(source.owner(), source.repo(), source.branch())
            .await
        {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed to fetch tree: {}", e);
                continue;
            }
        };

        for entry in tree.tree.iter().filter(|e| source.matches_pattern(&e.path)) {
            match fetch_skill(&client, source, &entry.path, repo.stargazers_count).await {
                Ok(meta) => {
                    let score = scorer.calculate(&meta);
                    log::debug!("Skill {} scored {:.1}", meta.skill_id, score);
                    all_skills.push((meta, score));
                }
                Err(e) => log::warn!("Failed to fetch {}: {}", entry.path, e),
            }
        }
    }

    let filtered = scorer.filter_above_threshold(all_skills, 60.0);
    let top = scorer.sort_and_limit(filtered, 50);

    let skills: HashMap<String, rulesify::models::Skill> = top
        .into_iter()
        .map(|(meta, score)| (meta.skill_id.clone(), meta.to_skill(score)))
        .collect();

    log::info!("Generated {} skills", skills.len());

    let gen = RegistryGenerator::new(1);
    let registry = gen.generate(skills);
    gen.write(&registry, std::path::Path::new("registry.toml"))?;

    log::info!("Written to registry.toml");
    Ok(())
}
