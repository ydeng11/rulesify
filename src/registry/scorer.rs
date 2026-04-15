use crate::models::SkillMetadata;

pub struct Scorer {
    stars_weight: f32,
    stars_cap: u32,
}

impl Default for Scorer {
    fn default() -> Self {
        Self {
            stars_weight: 0.50,
            stars_cap: 10000,
        }
    }
}

impl Scorer {
    pub fn calculate(&self, meta: &SkillMetadata) -> f32 {
        let stars_norm = (meta.stars as f32 / self.stars_cap as f32).min(1.0);
        let base_score = stars_norm * self.stars_weight;
        let activity_bonus = 0.30;
        let domain_bonus = 0.20;

        (base_score + activity_bonus + domain_bonus) * 100.0
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
