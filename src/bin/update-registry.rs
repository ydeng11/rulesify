use anyhow::Result;
use clap::Parser;
use rulesify::{
    llm::{Classifier, SkillClassification},
    models::{InstallAction, Registry, RepoMetrics, SkillMetadata},
    registry::{GitHubClient, RegistryGenerator, Scorer, SkillParser, SourceRepo},
};
use std::collections::HashMap;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(name = "update-registry")]
#[command(about = "Update skill registry from GitHub sources")]
struct Args {
    #[arg(
        long,
        help = "Force re-classification of all skills (ignore cached domain/tags)"
    )]
    force: bool,
}

async fn fetch_skill(
    client: &GitHubClient,
    source: SourceRepo,
    path: &str,
    repo_metrics: &RepoMetrics,
) -> Result<SkillMetadata> {
    let content = client
        .fetch_file(source.owner(), source.repo(), path)
        .await?;
    let parsed = SkillParser::parse(&content)?;
    let skill_id = source.parse_skill_id(path).unwrap_or("unknown".into());
    let context_size = SkillParser::estimate_context_size(&content);
    let folder = source.parse_skill_folder(path).unwrap_or_default();

    let source_url = format!(
        "https://github.com/{}/skills/tree/{}/{}",
        source.owner(),
        source.branch(),
        folder
    );

    Ok(SkillMetadata {
        skill_id,
        name: parsed.name,
        description: parsed.description,
        source_repo: source.full_name(),
        source_folder: folder.clone(),
        source_url,
        tags: parsed.tags,
        stars: repo_metrics.stars,
        context_size,
        domain: String::new(),
        last_updated: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        install_action: InstallAction::Copy { folder },
    })
}

fn load_cached_registry(path: &Path) -> HashMap<String, (String, Vec<String>)> {
    if !path.exists() {
        return HashMap::new();
    }

    let content = std::fs::read_to_string(path).unwrap_or_default();
    let registry: Registry = toml::from_str(&content).unwrap_or_else(|_| Registry {
        version: 0,
        updated: String::new(),
        skills: HashMap::new(),
    });

    registry
        .skills
        .into_iter()
        .filter(|(_, skill)| !skill.domain.is_empty())
        .map(|(id, skill)| (id, (skill.domain, skill.tags)))
        .collect()
}

fn apply_classification(meta: &mut SkillMetadata, classification: &SkillClassification) {
    meta.domain = classification.domain.to_string();
    meta.tags = classification.tags.clone();
}

fn apply_cache(meta: &mut SkillMetadata, cached: &HashMap<String, (String, Vec<String>)>) -> bool {
    if let Some((domain, tags)) = cached.get(&meta.skill_id) {
        if !domain.is_empty() {
            meta.domain = domain.clone();
            meta.tags = tags.clone();
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    let token = std::env::var("GITHUB_TOKEN").ok();
    let client = if let Some(t) = token {
        log::info!("Using authenticated GitHub API");
        GitHubClient::with_token(Some(t))
    } else {
        log::warn!("No GITHUB_TOKEN set - using unauthenticated API (60 requests/hr rate limit)");
        GitHubClient::new()
    };
    let scorer = Scorer::default();

    let registry_path = Path::new("registry.toml");
    let cached = if args.force {
        log::info!("Force flag set - skipping cache");
        HashMap::new()
    } else {
        log::info!("Loading cached registry");
        load_cached_registry(registry_path)
    };

    let mut all_skills: Vec<(SkillMetadata, f32)> = vec![];

    for source in SourceRepo::all() {
        log::info!("Fetching from {}", source.full_name());

        let repo_metrics = match client
            .fetch_repo_metrics(source.owner(), source.repo())
            .await
        {
            Ok(m) => m,
            Err(e) => {
                log::warn!(
                    "Failed to fetch repo metrics for {}: {}",
                    source.full_name(),
                    e
                );
                RepoMetrics::default()
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
            match fetch_skill(&client, source, &entry.path, &repo_metrics).await {
                Ok(meta) => {
                    let score = scorer.calculate_for_skill(&meta, &repo_metrics);
                    log::debug!("Skill {} scored {:.1}", meta.skill_id, score);
                    all_skills.push((meta, score));
                }
                Err(e) => log::warn!("Failed to fetch {}: {}", entry.path, e),
            }
        }
    }

    let filtered = scorer.filter_above_threshold(all_skills, 30.0);
    let top = scorer.sort_and_limit(filtered, 50);

    let mut pending_skills: HashMap<String, (SkillMetadata, f32)> = HashMap::new();
    let mut skills_to_classify: Vec<(String, String)> = vec![];
    let mut final_skills: HashMap<String, rulesify::models::Skill> = HashMap::new();

    for (mut meta, score) in top {
        let was_cached = apply_cache(&mut meta, &cached);

        if was_cached {
            log::debug!("Using cached classification for {}", meta.skill_id);
            final_skills.insert(meta.skill_id.clone(), meta.to_skill(score));
        } else {
            skills_to_classify.push((meta.skill_id.clone(), meta.description.clone()));
            pending_skills.insert(meta.skill_id.clone(), (meta, score));
        }
    }

    if !skills_to_classify.is_empty() {
        log::info!(
            "Classifying {} skills with LLM (model: {})",
            skills_to_classify.len(),
            std::env::var("OPENROUTER_MODEL")
                .unwrap_or_else(|_| "anthropic/claude-3.5-haiku".to_string())
        );

        let classifier = Classifier::from_env()?;
        let classifications = classifier.classify(skills_to_classify).await?;

        for (skill_id, classification) in classifications {
            if let Some((mut meta, score)) = pending_skills.remove(&skill_id) {
                apply_classification(&mut meta, &classification);
                final_skills.insert(skill_id, meta.to_skill(score));
            }
        }

        for (skill_id, (meta, score)) in pending_skills {
            log::warn!("Skill '{}' not classified, using fallback", skill_id);
            let mut updated_meta = meta;
            updated_meta.domain = "development".to_string();
            updated_meta.tags = vec![];
            final_skills.insert(skill_id, updated_meta.to_skill(score));
        }
    }

    log::info!("Generated {} skills", final_skills.len());

    let gen = RegistryGenerator::new(1);
    let registry = gen.generate(final_skills);
    gen.write(&registry, registry_path)?;

    log::info!("Written to registry.toml");
    Ok(())
}
