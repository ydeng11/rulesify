use crate::llm::client::OpenRouterClient;
use crate::llm::prompt::{build_prompts, build_user_prompt};
use crate::models::Domain;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

const BATCH_SIZE: usize = 25;

#[derive(Debug, Clone)]
pub struct SkillClassification {
    pub domain: Domain,
    pub tags: Vec<String>,
}

pub type ClassificationResult = HashMap<String, SkillClassification>;

#[derive(Debug, Deserialize)]
struct BatchResponse {
    #[serde(flatten)]
    skills: HashMap<String, SkillClassificationRaw>,
}

#[derive(Debug, Deserialize)]
struct SkillClassificationRaw {
    domain: String,
    tags: Vec<String>,
}

pub struct Classifier {
    client: OpenRouterClient,
    system_prompt: String,
}

impl Classifier {
    pub fn from_env() -> Result<Self> {
        let client = OpenRouterClient::from_env()?;
        let (system_prompt, _) = build_prompts();
        Ok(Self {
            client,
            system_prompt,
        })
    }

    pub fn new(client: OpenRouterClient) -> Self {
        let (system_prompt, _) = build_prompts();
        Self {
            client,
            system_prompt,
        }
    }

    pub fn model(&self) -> &str {
        self.client.model()
    }

    pub async fn classify(&self, skills: Vec<(String, String)>) -> Result<ClassificationResult> {
        let mut results = ClassificationResult::new();

        for batch in skills.chunks(BATCH_SIZE) {
            let batch_results = self.classify_batch(batch).await?;
            results.extend(batch_results);
        }

        Ok(results)
    }

    async fn classify_batch(&self, batch: &[(String, String)]) -> Result<ClassificationResult> {
        if batch.is_empty() {
            return Ok(ClassificationResult::new());
        }

        log::info!(
            "Classifying batch of {} skills using model {}",
            batch.len(),
            self.client.model()
        );

        let user_prompt = build_user_prompt(batch);
        let response = self
            .client
            .classify_batch(&self.system_prompt, &user_prompt)
            .await?;

        self.parse_response(batch, &response)
    }

    fn parse_response(
        &self,
        batch: &[(String, String)],
        response: &str,
    ) -> Result<ClassificationResult> {
        let mut results = ClassificationResult::new();

        let cleaned = response
            .trim()
            .strip_prefix("```json")
            .unwrap_or(response)
            .strip_prefix("```")
            .unwrap_or(response)
            .trim()
            .strip_suffix("```")
            .unwrap_or(response.trim());

        let parsed: BatchResponse = serde_json::from_str(cleaned)
            .with_context(|| format!("Failed to parse LLM response: {}", response))?;

        for (skill_id, classification) in parsed.skills {
            let domain = Domain::from_str(&classification.domain).unwrap_or_else(|_| {
                log::warn!(
                    "Invalid domain '{}' for skill '{}', using fallback",
                    classification.domain,
                    skill_id
                );
                Domain::default_fallback()
            });

            let tags = classification
                .tags
                .iter()
                .map(|t| t.to_lowercase().replace(' ', "-"))
                .take(3)
                .collect();

            results.insert(skill_id.clone(), SkillClassification { domain, tags });
        }

        for (skill_id, _) in batch {
            if !results.contains_key(skill_id) {
                log::warn!(
                    "Skill '{}' missing from LLM response, using fallback",
                    skill_id
                );
                results.insert(
                    skill_id.clone(),
                    SkillClassification {
                        domain: Domain::default_fallback(),
                        tags: vec![],
                    },
                );
            }
        }

        Ok(results)
    }
}
