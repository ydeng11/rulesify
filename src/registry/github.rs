use crate::models::RepoMetrics;
use crate::utils::{Result, RulesifyError};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::Deserialize;
use std::time::Duration;
use tokio::time::sleep;

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

#[derive(Debug, Clone, Deserialize)]
pub struct ContentEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub content_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileContent {
    pub content: Option<String>,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub state: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Contributor {
    pub login: String,
    pub contributions: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitRef {
    pub sha: String,
}

pub struct GitHubClient {
    http: reqwest::Client,
    pub token: Option<String>,
}

impl Default for GitHubClient {
    fn default() -> Self {
        Self::with_token(None)
    }
}

impl GitHubClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_token(token: Option<String>) -> Self {
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

    async fn fetch_with_retry(&self, url: &str, max_retries: u32) -> Result<reqwest::Response> {
        let mut retries = 0;
        let mut delay = Duration::from_secs(1);

        loop {
            let resp = self
                .request(url)
                .send()
                .await
                .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

            if resp.status().is_success() {
                return Ok(resp);
            }

            let status = resp.status();
            let should_retry =
                (status.as_u16() == 429 || status.as_u16() == 403) && retries < max_retries;

            if should_retry {
                log::warn!(
                    "GitHub API returned HTTP {}, retrying in {} seconds (retry {} of {})",
                    status.as_u16(),
                    delay.as_secs(),
                    retries + 1,
                    max_retries
                );
                sleep(delay).await;
                delay = Duration::from_secs(delay.as_secs() * 2);
                retries += 1;
                continue;
            }

            return Err(RulesifyError::GitHubApi(format!("HTTP {}", status)).into());
        }
    }

    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<RepoInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = self.fetch_with_retry(&url, 3).await?;

        resp.json::<RepoInfo>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_tree(&self, owner: &str, repo: &str, branch: &str) -> Result<TreeResponse> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/git/trees/{}?recursive=1",
            owner, repo, branch
        );
        let resp = self.fetch_with_retry(&url, 3).await?;

        resp.json::<TreeResponse>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    async fn fetch_raw_with_retry(
        &self,
        url: &str,
        accept_header: &str,
        max_retries: u32,
    ) -> Result<String> {
        let mut retries = 0;
        let mut delay = Duration::from_secs(1);

        loop {
            let mut req = self
                .http
                .get(url)
                .header("Accept", accept_header)
                .header("User-Agent", "rulesify");

            if let Some(auth) = self.auth_header() {
                req = req.header("Authorization", auth);
            }

            let resp = req
                .send()
                .await
                .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

            if resp.status().is_success() {
                return resp
                    .text()
                    .await
                    .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into());
            }

            let status = resp.status();
            let should_retry =
                (status.as_u16() == 429 || status.as_u16() == 403) && retries < max_retries;

            if should_retry {
                log::warn!(
                    "GitHub API returned HTTP {} for raw content, retrying in {} seconds (retry {} of {})",
                    status.as_u16(),
                    delay.as_secs(),
                    retries + 1,
                    max_retries
                );
                sleep(delay).await;
                delay = Duration::from_secs(delay.as_secs() * 2);
                retries += 1;
                continue;
            }

            return Err(RulesifyError::GitHubApi(format!("HTTP {}", status)).into());
        }
    }

    pub async fn fetch_file(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        );
        self.fetch_raw_with_retry(&url, "application/vnd.github.v3.raw", 3)
            .await
    }

    pub fn contents_url(&self, owner: &str, repo: &str, path: &str) -> String {
        format!(
            "https://api.github.com/repos/{}/{}/contents/{}",
            owner, repo, path
        )
    }

    pub async fn list_folder(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<Vec<ContentEntry>> {
        let url = self.contents_url(owner, repo, path);
        let resp = self.fetch_with_retry(&url, 3).await?;

        resp.json::<Vec<ContentEntry>>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_file_raw(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}/main/{}",
            owner, repo, path
        );

        let mut retries = 0;
        let mut delay = Duration::from_secs(1);

        loop {
            let resp = self
                .http
                .get(&url)
                .header("User-Agent", "rulesify")
                .send()
                .await
                .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

            if resp.status().is_success() {
                return resp
                    .text()
                    .await
                    .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into());
            }

            let status = resp.status();
            let should_retry = (status.as_u16() == 429 || status.as_u16() == 403) && retries < 3;

            if should_retry {
                log::warn!(
                    "GitHub API returned HTTP {} for raw file, retrying in {} seconds (retry {} of 3)",
                    status.as_u16(),
                    delay.as_secs(),
                    retries + 1
                );
                sleep(delay).await;
                delay = Duration::from_secs(delay.as_secs() * 2);
                retries += 1;
                continue;
            }

            return Err(RulesifyError::GitHubApi(format!("HTTP {}", status)).into());
        }
    }

    pub async fn fetch_issues(
        &self,
        owner: &str,
        repo: &str,
        since: DateTime<Utc>,
    ) -> Result<Vec<Issue>> {
        let since_str = since.format("%Y-%m-%dT%H:%M:%SZ");
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues?state=all&since={}&per_page=100",
            owner, repo, since_str
        );
        let resp = self.fetch_with_retry(&url, 3).await?;

        resp.json::<Vec<Issue>>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_contributors(&self, owner: &str, repo: &str) -> Result<Vec<Contributor>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/contributors?per_page=100",
            owner, repo
        );
        let resp = self.fetch_with_retry(&url, 3).await?;

        resp.json::<Vec<Contributor>>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_commit_for_path(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
    ) -> Result<String> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/commits?path={}&per_page=1",
            owner, repo, path
        );
        let resp = self.fetch_with_retry(&url, 3).await?;

        let commits: Vec<CommitRef> = resp
            .json()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;

        Ok(commits
            .first()
            .map(|c| c.sha.clone())
            .ok_or_else(|| RulesifyError::GitHubApi("No commits found for path".into()))?)
    }

    pub async fn fetch_repo_metrics(&self, owner: &str, repo: &str) -> Result<RepoMetrics> {
        let repo_info = self.fetch_repo(owner, repo).await?;

        let three_months_ago = Utc::now() - ChronoDuration::days(90);
        let issues = self.fetch_issues(owner, repo, three_months_ago).await?;

        let mut issues_open_3mo = 0u32;
        let mut issues_closed_3mo = 0u32;
        for issue in issues.iter() {
            if issue.created_at >= three_months_ago {
                if issue.state == "open" {
                    issues_open_3mo += 1;
                } else if issue.state == "closed" {
                    issues_closed_3mo += 1;
                }
            }
        }

        let contributors = self.fetch_contributors(owner, repo).await?;
        let contributors_total = contributors.len() as u32;
        let contributors_active_3mo =
            contributors.iter().filter(|c| c.contributions > 0).count() as u32;

        Ok(RepoMetrics {
            stars: repo_info.stargazers_count,
            pushed_at: repo_info.pushed_at,
            issues_open_3mo,
            issues_closed_3mo,
            contributors_total,
            contributors_active_3mo,
        })
    }
}
