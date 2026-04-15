use crate::models::{RepoMetrics, SkillMetadata};
use chrono::{DateTime, Utc};

pub struct Scorer {
    stars_weight: f32,
    stars_cap: u32,
    recency_weight: f32,
    issue_weight: f32,
    contributor_weight: f32,
}

impl Default for Scorer {
    fn default() -> Self {
        Self {
            stars_weight: 40.0,
            stars_cap: 10000,
            recency_weight: 30.0,
            issue_weight: 20.0,
            contributor_weight: 10.0,
        }
    }
}

impl Scorer {
    pub fn calculate(&self, metrics: &RepoMetrics) -> f32 {
        let stars_score = self.stars_score(metrics.stars);
        let recency_score = self.recency_score(&metrics.pushed_at);
        let issue_score = self.issue_score(metrics);
        let contributor_score = self.contributor_score(metrics);

        stars_score + recency_score + issue_score + contributor_score
    }

    fn stars_score(&self, stars: u32) -> f32 {
        (stars as f32 / self.stars_cap as f32).min(1.0) * self.stars_weight
    }

    fn recency_score(&self, pushed_at: &DateTime<Utc>) -> f32 {
        let days = (Utc::now() - *pushed_at).num_days();
        if days < 30 {
            self.recency_weight
        } else if days < 90 {
            self.recency_weight * 0.5
        } else {
            self.recency_weight * 0.1
        }
    }

    fn issue_score(&self, metrics: &RepoMetrics) -> f32 {
        let total = metrics.issues_open_3mo + metrics.issues_closed_3mo;
        if total == 0 {
            return self.issue_weight * 0.5;
        }
        let resolution_rate = metrics.issues_closed_3mo as f32 / total as f32;
        resolution_rate * self.issue_weight
    }

    fn contributor_score(&self, metrics: &RepoMetrics) -> f32 {
        if metrics.contributors_total == 0 {
            return self.contributor_weight * 0.5;
        }
        let activity_rate =
            metrics.contributors_active_3mo as f32 / metrics.contributors_total as f32;
        activity_rate * self.contributor_weight
    }

    pub fn calculate_for_skill(&self, _meta: &SkillMetadata, repo_metrics: &RepoMetrics) -> f32 {
        self.calculate(repo_metrics)
    }

    pub fn filter_above_threshold(
        &self,
        skills: Vec<(SkillMetadata, f32)>,
        min: f32,
    ) -> Vec<(SkillMetadata, f32)> {
        skills.into_iter().filter(|(_, s)| *s >= min).collect()
    }

    pub fn sort_and_limit(
        &self,
        skills: Vec<(SkillMetadata, f32)>,
        limit: usize,
    ) -> Vec<(SkillMetadata, f32)> {
        let mut sorted = skills;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(limit);
        sorted
    }
}
