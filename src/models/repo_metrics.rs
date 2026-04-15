use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMetrics {
    pub stars: u32,
    pub pushed_at: DateTime<Utc>,
    pub issues_open_3mo: u32,
    pub issues_closed_3mo: u32,
    pub contributors_total: u32,
    pub contributors_active_3mo: u32,
}

impl Default for RepoMetrics {
    fn default() -> Self {
        Self {
            stars: 0,
            pushed_at: Utc::now(),
            issues_open_3mo: 0,
            issues_closed_3mo: 0,
            contributors_total: 0,
            contributors_active_3mo: 0,
        }
    }
}
