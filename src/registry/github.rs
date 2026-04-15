use crate::utils::{Result, RulesifyError};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize)]
pub struct RepoInfo {
    pub full_name: String,
    pub stargazers_count: u32,
    pub pushed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeEntry {
    pub path: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TreeResponse {
    pub tree: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitInfo {
    pub commit: CommitDetails,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitDetails {
    pub author: Option<CommitAuthor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitAuthor {
    pub date: DateTime<Utc>,
}

pub struct GitHubClient {
    http: reqwest::Client,
    pub token: Option<String>,
}

impl GitHubClient {
    pub fn new(token: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("rulesify")
            .build()
            .unwrap();

        Self { http, token }
    }

    fn auth_header(&self) -> Option<String> {
        self.token.as_ref().map(|t| format!("Bearer {}", t))
    }

    fn request(&self, url: &str) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .get(url)
            .header("Accept", "application/vnd.github.v3+json");

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        req
    }

    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<RepoInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = self
            .request(&url)
            .send()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RulesifyError::GitHubApi(format!("HTTP {}", resp.status())).into());
        }

        resp.json::<RepoInfo>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_tree(&self, owner: &str, repo: &str, branch: &str) -> Result<TreeResponse> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, repo, branch
        );
        let resp = self
            .request(&url)
            .send()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RulesifyError::GitHubApi(format!("HTTP {}", resp.status())).into());
        }

        resp.json::<TreeResponse>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_file(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );
        let mut req = self
            .http
            .get(&url)
            .header("Accept", "application/vnd.github.v3.raw")
            .header("User-Agent", "rulesify");

        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }

        let resp = req
            .send()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(RulesifyError::GitHubApi(format!("HTTP {}", resp.status())).into());
        }

        resp.text()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }
}
