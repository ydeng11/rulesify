# Skills Registry Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a pure metadata registry catalog that fetches skill info from GitHub repos, scores them, and generates registry.toml with install instructions.

**Architecture:** Registry is a catalog (no local copies). Each skill has metadata (stars, context_size, domain, last_updated) and install_action (copy/command types). GitHub Actions runs weekly updates.

**Tech Stack:** Rust (tokio async, reqwest), serde_yaml, chrono, GitHub API v3

---

## Registry Data Model

```toml
[skills.tdd]
name = "Test-Driven Development"
description = "Write tests before implementation"
source_url = "https://github.com/mattpocock/skills/tree/main/tdd"
stars = 1500
context_size = 2400          # approximate tokens
domain = "development"
last_updated = "2026-04-10"
tags = ["testing", "development"]
install_action = { type = "copy", path = "tdd/SKILL.md" }

[skills.gsd]
name = "Get Shit Done"
description = "Project management for solo developers"
source_url = "https://github.com/gsd-build/get-shit-done"
stars = 3200
context_size = 15000
domain = "planning"
last_updated = "2026-04-08"
tags = ["project-management", "planning"]
install_action = { type = "command", value = "git clone https://github.com/gsd-build/get-shit-done ~/.agents/skills/gsd" }
```

---

## Task 1: Add Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Add dependencies**

Add after `log = "0.4"`:

```toml
serde_yaml = "0.9"
serde_json = "1.0"
```

**Step 2: Verify**

Run: `cargo check`
Expected: PASS

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: add serde_yaml, serde_json for registry"
```

---

## Task 2: Create InstallAction Model

**Files:**
- Create: `src/models/install_action.rs`
- Modify: `src/models/mod.rs`
- Create: `src/models/install_action_tests.rs`

**Step 1: Write failing test**

Create `src/models/install_action_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::models::InstallAction;

    #[test]
    fn test_copy_action() {
        let action = InstallAction::Copy { path: "tdd/SKILL.md".to_string() };
        assert!(action.is_simple());
        assert_eq!(action.install_command("https://github.com/test/skills"), Some("rulesify skill fetch https://github.com/test/skills/tdd/SKILL.md".to_string()));
    }

    #[test]
    fn test_command_action() {
        let action = InstallAction::Command { value: "git clone https://example.com ~/.agents/skills/foo".to_string() };
        assert!(!action.is_simple());
        assert_eq!(action.install_command("https://github.com/test"), Some("git clone https://example.com ~/.agents/skills/foo".to_string()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test install_action_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/models/install_action.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InstallAction {
    #[serde(rename = "copy")]
    Copy { path: String },
    #[serde(rename = "command")]
    Command { value: String },
}

impl InstallAction {
    pub fn is_simple(&self) -> bool {
        matches!(self, InstallAction::Copy { .. })
    }

    pub fn install_command(&self, source_url: &str) -> Option<String> {
        match self {
            InstallAction::Copy { path } => {
                let file_url = format!("{}/{}", source_url.replace("/tree/", "/blob/"), path);
                Some(format!("rulesify skill fetch {}", file_url))
            },
            InstallAction::Command { value } => Some(value.clone()),
        }
    }

    pub fn default_copy(skill_path: &str) -> Self {
        InstallAction::Copy { path: skill_path.to_string() }
    }
}
```

**Step 4: Update mod.rs**

Modify `src/models/mod.rs`:

```rust
pub mod skill;
pub mod registry;
pub mod context;
pub mod config;
pub mod install_action;

#[cfg(test)]
mod skill_tests;
mod install_action_tests;

pub use skill::Skill;
pub use registry::Registry;
pub use context::ProjectContext;
pub use config::ProjectConfig;
pub use install_action::InstallAction;
```

**Step 5: Run test**

Run: `cargo test install_action_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/models/install_action.rs src/models/mod.rs src/models/install_action_tests.rs
git commit -m "feat: add InstallAction enum for copy/command install types"
```

---

## Task 3: Update Skill Model

**Files:**
- Modify: `src/models/skill.rs`
- Modify: `src/models/skill_tests.rs`

**Step 1: Write failing test**

Add to `src/models/skill_tests.rs`:

```rust
#[test]
fn test_skill_with_new_fields() {
    let skill = Skill {
        name: "TDD".to_string(),
        description: "Test driven development".to_string(),
        source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
        stars: 1500,
        context_size: 2400,
        domain: "development".to_string(),
        last_updated: "2026-04-10".to_string(),
        tags: vec!["testing".to_string()],
        install_action: Some(InstallAction::Copy { path: "tdd/SKILL.md".to_string() }),
        score: Some(85.0),
    };
    assert_eq!(skill.stars, 1500);
    assert!(skill.install_action.unwrap().is_simple());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test skill_tests::test_skill_with_new_fields`
Expected: FAIL

**Step 3: Update Skill struct**

Modify `src/models/skill.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::models::InstallAction;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub source_url: String,
    pub stars: u32,
    #[serde(default)]
    pub context_size: u32,
    #[serde(default)]
    pub domain: String,
    pub last_updated: String,
    #[serde(default)]
    pub tags: Vec<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_action: Option<InstallAction>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
}

impl Skill {
    pub fn matches_tags(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }

    pub fn matches_domain(&self, domain: &str) -> bool {
        self.domain == domain
    }
}
```

**Step 4: Run test**

Run: `cargo test skill_tests::test_skill_with_new_fields`
Expected: PASS

**Step 5: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/models/skill.rs src/models/skill_tests.rs
git commit -m "feat: update Skill model with stars, context_size, domain, install_action"
```

---

## Task 4: Update Registry Model

**Files:**
- Modify: `src/models/registry.rs`
- Modify: `src/models/registry_tests.rs` (create if needed)

**Step 1: Write failing test**

Create `src/models/registry_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::models::{Registry, Skill, InstallAction};
    use std::collections::HashMap;

    #[test]
    fn test_registry_new_format() {
        let mut skills = HashMap::new();
        skills.insert("tdd".to_string(), Skill {
            name: "TDD".to_string(),
            description: "Test driven development methodology".to_string(),
            source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
            stars: 1500,
            context_size: 2400,
            domain: "development".to_string(),
            last_updated: "2026-04-10".to_string(),
            tags: vec!["testing".to_string()],
            install_action: Some(InstallAction::Copy { path: "tdd/SKILL.md".to_string() }),
            score: Some(85.0),
        });
        
        let registry = Registry {
            version: 1,
            updated: "2026-04-14".to_string(),
            skills,
        };
        
        assert_eq!(registry.skills.len(), 1);
        assert!(registry.get_skill("tdd").is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test registry_tests`
Expected: FAIL (missing fields)

**Step 3: Update Registry struct**

Modify `src/models/registry.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::Skill;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub updated: String,
    pub skills: HashMap<String, Skill>,
}

impl Registry {
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }
    
    pub fn filter_by_domain(&self, domain: &str) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_domain(domain))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_tags(tags))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn all_skills(&self) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
```

**Step 4: Update mod.rs**

Modify `src/models/mod.rs`:

```rust
pub mod skill;
pub mod registry;
pub mod context;
pub mod config;
pub mod install_action;

#[cfg(test)]
mod skill_tests;
mod install_action_tests;
mod registry_tests;

pub use skill::Skill;
pub use registry::Registry;
pub use context::ProjectContext;
pub use config::ProjectConfig;
pub use install_action::InstallAction;
```

**Step 5: Run test**

Run: `cargo test registry_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/models/registry.rs src/models/mod.rs src/models/registry_tests.rs
git commit -m "feat: update Registry model for new skill format"
```

---

## Task 5: Create SkillMetadata Model

**Files:**
- Create: `src/models/skill_metadata.rs`
- Modify: `src/models/mod.rs`
- Create: `src/models/skill_metadata_tests.rs`

**Step 1: Write failing test**

Create `src/models/skill_metadata_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::models::SkillMetadata;
    use crate::models::InstallAction;

    #[test]
    fn test_metadata_creation() {
        let meta = SkillMetadata {
            skill_id: "tdd".to_string(),
            name: "Test-Driven Development".to_string(),
            description: "Write tests before implementation".to_string(),
            source_repo: "mattpocock/skills".to_string(),
            source_path: "tdd/SKILL.md".to_string(),
            source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
            tags: vec!["testing".to_string()],
            stars: 1500,
            context_size: 2400,
            domain: "development".to_string(),
            last_updated: "2026-04-10".to_string(),
            install_action: InstallAction::Copy { path: "tdd/SKILL.md".to_string() },
        };
        assert_eq!(meta.skill_id, "tdd");
        assert!(meta.install_action.is_simple());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test skill_metadata_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/models/skill_metadata.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::models::{Skill, InstallAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub source_repo: String,
    pub source_path: String,
    pub source_url: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub stars: u32,
    #[serde(default)]
    pub context_size: u32,
    #[serde(default)]
    pub domain: String,
    pub last_updated: String,
    pub install_action: InstallAction,
}

impl SkillMetadata {
    pub fn to_skill(&self, score: f32) -> Skill {
        Skill {
            name: self.name.clone(),
            description: self.description.clone(),
            source_url: self.source_url.clone(),
            stars: self.stars,
            context_size: self.context_size,
            domain: self.domain.clone(),
            last_updated: self.last_updated.clone(),
            tags: self.tags.clone(),
            install_action: Some(self.install_action.clone()),
            score: Some(score),
        }
    }
}
```

**Step 4: Update mod.rs**

Modify `src/models/mod.rs`:

```rust
pub mod skill;
pub mod registry;
pub mod context;
pub mod config;
pub mod install_action;
pub mod skill_metadata;

#[cfg(test)]
mod skill_tests;
mod install_action_tests;
mod registry_tests;
mod skill_metadata_tests;

pub use skill::Skill;
pub use registry::Registry;
pub use context::ProjectContext;
pub use config::ProjectConfig;
pub use install_action::InstallAction;
pub use skill_metadata::SkillMetadata;
```

**Step 5: Run test**

Run: `cargo test skill_metadata_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/models/skill_metadata.rs src/models/mod.rs src/models/skill_metadata_tests.rs
git commit -m "feat: add SkillMetadata for raw skill data from GitHub"
```

---

## Task 6: Create SourceRepo Enum

**Files:**
- Create: `src/registry/source.rs`
- Modify: `src/registry/mod.rs`
- Create: `src/registry/source_tests.rs`

**Step 1: Write failing test**

Create `src/registry/source_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::registry::SourceRepo;

    #[test]
    fn test_all_sources() {
        let sources = SourceRepo::all();
        assert!(sources.len() >= 5);
    }

    #[test]
    fn test_source_properties() {
        let anthropic = SourceRepo::AnthropicSkills;
        assert_eq!(anthropic.owner(), "anthropics");
        assert_eq!(anthropic.repo(), "skills");
        assert_eq!(anthropic.branch(), "main");
        assert_eq!(anthropic.skill_pattern(), "skills/*/SKILL.md");
    }

    #[test]
    fn test_parse_skill_id() {
        let anthropic = SourceRepo::AnthropicSkills;
        let id = anthropic.parse_skill_id("skills/tdd/SKILL.md");
        assert_eq!(id, Some("tdd".to_string()));
        
        let mattpocock = SourceRepo::MattPocockSkills;
        let id = mattpocock.parse_skill_id("brainstorming/SKILL.md");
        assert_eq!(id, Some("brainstorming".to_string()));
        
        let openai = SourceRepo::OpenAISkills;
        let id = openai.parse_skill_id("skills/.curated/gh-fix-ci/SKILL.md");
        assert_eq!(id, Some("gh-fix-ci".to_string()));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test source_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/registry/source.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceRepo {
    AnthropicSkills,
    OpenAISkillsCurated,
    OpenAISkillsSystem,
    OpenAISkillsExperimental,
    MattPocockSkills,
    MiniMaxSkills,
}

impl SourceRepo {
    pub fn all() -> Vec<Self> {
        vec![
            SourceRepo::AnthropicSkills,
            SourceRepo::OpenAISkillsCurated,
            SourceRepo::MattPocockSkills,
            SourceRepo::MiniMaxSkills,
        ]
    }

    pub fn owner(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "anthropics",
            SourceRepo::OpenAISkillsCurated | SourceRepo::OpenAISkillsSystem | SourceRepo::OpenAISkillsExperimental => "openai",
            SourceRepo::MattPocockSkills => "mattpocock",
            SourceRepo::MiniMaxSkills => "MiniMax-AI",
        }
    }

    pub fn repo(&self) -> &'static str {
        "skills"
    }

    pub fn branch(&self) -> &'static str {
        "main"
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner(), self.repo())
    }

    pub fn skill_pattern(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "skills/*/SKILL.md",
            SourceRepo::OpenAISkillsCurated => "skills/.curated/*/SKILL.md",
            SourceRepo::OpenAISkillsSystem => "skills/.system/*/SKILL.md",
            SourceRepo::OpenAISkillsExperimental => "skills/.experimental/*/SKILL.md",
            SourceRepo::MattPocockSkills => "*/SKILL.md",
            SourceRepo::MiniMaxSkills => "skills/*/SKILL.md",
        }
    }

    pub fn domain(&self) -> &'static str {
        match self {
            SourceRepo::AnthropicSkills => "development",
            SourceRepo::OpenAISkillsCurated => "general",
            SourceRepo::OpenAISkillsSystem => "system",
            SourceRepo::OpenAISkillsExperimental => "experimental",
            SourceRepo::MattPocockSkills => "development",
            SourceRepo::MiniMaxSkills => "general",
        }
    }

    pub fn parse_skill_id(&self, path: &str) -> Option<String> {
        let parts: Vec<&str> = path.split('/').collect();
        
        match self {
            SourceRepo::AnthropicSkills | SourceRepo::MiniMaxSkills => {
                if parts.len() >= 3 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[2].to_string())
                } else {
                    None
                }
            },
            SourceRepo::OpenAISkillsCurated | SourceRepo::OpenAISkillsSystem | SourceRepo::OpenAISkillsExperimental => {
                if parts.len() >= 4 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[3].to_string())
                } else {
                    None
                }
            },
            SourceRepo::MattPocockSkills => {
                if parts.len() >= 2 && parts.last() == Some(&"SKILL.md") {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            },
        }
    }

    pub fn matches_pattern(&self, path: &str) -> bool {
        path.ends_with("SKILL.md") && self.parse_skill_id(path).is_some()
    }
}
```

**Step 4: Update mod.rs**

Modify `src/registry/mod.rs`:

```rust
pub mod data;
pub mod fetch;
pub mod cache;
pub mod source;

#[cfg(test)]
mod data_tests;
mod source_tests;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
pub use source::SourceRepo;
```

**Step 5: Run test**

Run: `cargo test source_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/registry/source.rs src/registry/mod.rs src/registry/source_tests.rs
git commit -m "feat: add SourceRepo enum for GitHub skill repos"
```

---

## Task 7: Create GitHub API Client

**Files:**
- Create: `src/registry/github.rs`
- Modify: `src/registry/mod.rs`
- Modify: `src/utils/error.rs`
- Create: `src/registry/github_tests.rs`

**Step 1: Write failing test**

Create `src/registry/github_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::registry::GitHubClient;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new(None);
        assert!(client.token.is_none());
        
        let client_with_token = GitHubClient::new(Some("test".to_string()));
        assert!(client_with_token.token.is_some());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test github_tests`
Expected: FAIL

**Step 3: Add error variant**

Modify `src/utils/error.rs`:

```rust
#[derive(Debug, Error)]
pub enum RulesifyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("Registry fetch error: {0}")]
    RegistryFetch(String),
    
    #[error("Config parse error: {0}")]
    ConfigParse(String),
    
    #[error("GitHub API error: {0}")]
    GitHubApi(String),
    
    #[error("Skill parse error: {0}")]
    SkillParse(String),
}
```

**Step 4: Write implementation**

Create `src/registry/github.rs`:

```rust
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
        let mut req = self.http.get(url)
            .header("Accept", "application/vnd.github.v3+json");
        
        if let Some(auth) = self.auth_header() {
            req = req.header("Authorization", auth);
        }
        
        req
    }

    pub async fn fetch_repo(&self, owner: &str, repo: &str) -> Result<RepoInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
        let resp = self.request(&url)
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
        let url = format!("https://api.github.com/repos/{}/{}git/trees/{}?recursive=1", owner, repo, branch);
        let resp = self.request(&url)
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
        let url = format!("https://api.github.com/repos/{}/{}contents/{}", owner, repo, path);
        let resp = self.http.get(&url)
            .header("Accept", "application/vnd.github.v3.raw")
            .header("User-Agent", "rulesify")
            .header("Authorization", self.auth_header().unwrap_or_default())
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

    pub async fn fetch_commits_since(&self, owner: &str, repo: &str, since: DateTime<Utc>) -> Result<Vec<CommitInfo>> {
        let url = format!("https://api.github.com/repos/{}/{}commits?since={}", owner, repo, since.to_rfc3339());
        let resp = self.request(&url)
            .send()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;
        
        if !resp.status().is_success() {
            return Err(RulesifyError::GitHubApi(format!("HTTP {}", resp.status())).into());
        }
        
        resp.json::<Vec<CommitInfo>>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }
}
```

**Step 5: Update mod.rs**

Modify `src/registry/mod.rs`:

```rust
pub mod data;
pub mod fetch;
pub mod cache;
pub mod source;
pub mod github;

#[cfg(test)]
mod data_tests;
mod source_tests;
mod github_tests;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
pub use source::SourceRepo;
pub use github::GitHubClient;
```

**Step 6: Run test**

Run: `cargo test github_tests`
Expected: PASS

**Step 7: Commit**

```bash
git add src/registry/github.rs src/registry/mod.rs src/registry/github_tests.rs src/utils/error.rs
git commit -m "feat: add GitHubClient for GitHub API v3"
```

---

## Task 8: Create SkillParser

**Files:**
- Create: `src/registry/parser.rs`
- Modify: `src/registry/mod.rs`
- Create: `src/registry/parser_tests.rs`

**Step 1: Write failing test**

Create `src/registry/parser_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::registry::SkillParser;

    #[test]
    fn test_parse_valid() {
        let content = "---\nname: tdd\ndescription: Test driven development methodology\ntags: [testing]\n---\n\n# TDD\n\nContent...";
        let parsed = SkillParser::parse(content).unwrap();
        assert_eq!(parsed.name, "tdd");
        assert!(parsed.description.len() >= 20);
    }

    #[test]
    fn test_parse_missing_frontmatter() {
        let content = "# No frontmatter";
        assert!(SkillParser::parse(content).is_err());
    }

    #[test]
    fn test_parse_short_description() {
        let content = "---\nname: test\ndescription: too short\n---\n\n# Test";
        assert!(SkillParser::parse(content).is_err());
    }

    #[test]
    fn test_estimate_context_size() {
        let content = "---\nname: test\ndescription: A long enough description here\n---\n\n# Test\n\nSome content\nMore lines\nEven more";
        let size = SkillParser::estimate_context_size(content);
        assert!(size > 0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test parser_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/registry/parser.rs`:

```rust
use crate::utils::{Result, RulesifyError};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ParsedSkill {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

pub struct SkillParser;

impl SkillParser {
    pub fn parse(content: &str) -> Result<ParsedSkill> {
        let frontmatter = Self::extract_frontmatter(content)?;
        let parsed: ParsedSkill = serde_yaml::from_str(&frontmatter)
            .map_err(|e| RulesifyError::SkillParse(format!("YAML error: {}", e)))?;
        
        Self::validate(&parsed)?;
        
        Ok(parsed)
    }

    fn extract_frontmatter(content: &str) -> Result<String> {
        if !content.starts_with("---") {
            return Err(RulesifyError::SkillParse("Missing frontmatter".into()).into());
        }
        
        let lines: Vec<&str> = content.lines().collect();
        let end = lines.iter().position(|l| l.trim() == "---").skip(1);
        
        if end.is_none() {
            return Err(RulesifyError::SkillParse("Unclosed frontmatter".into()).into());
        }
        
        Ok(lines[1..end.unwrap()].join("\n"))
    }

    fn validate(parsed: &ParsedSkill) -> Result<()> {
        if parsed.name.trim().is_empty() {
            return Err(RulesifyError::SkillParse("name required".into()).into());
        }
        if parsed.description.len() < 20 {
            return Err(RulesifyError::SkillParse("description too short".into()).into());
        }
        Ok(())
    }

    pub fn estimate_context_size(content: &str) -> u32 {
        let chars = content.len();
        let tokens = chars / 4;
        tokens as u32
    }
}
```

**Step 4: Update mod.rs**

Modify `src/registry/mod.rs`:

```rust
pub mod data;
pub mod fetch;
pub mod cache;
pub mod source;
pub mod github;
pub mod parser;

#[cfg(test)]
mod data_tests;
mod source_tests;
mod github_tests;
mod parser_tests;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
pub use source::SourceRepo;
pub use github::GitHubClient;
pub use parser::{SkillParser, ParsedSkill};
```

**Step 5: Run test**

Run: `cargo test parser_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/registry/parser.rs src/registry/mod.rs src/registry/parser_tests.rs
git commit -m "feat: add SkillParser for SKILL.md frontmatter"
```

---

## Task 9: Create Scorer

**Files:**
- Create: `src/registry/scorer.rs`
- Modify: `src/registry/mod.rs`
- Create: `src/registry/scorer_tests.rs`

**Step 1: Write failing test**

Create `src/registry/scorer_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::registry::Scorer;
    use crate::models::{SkillMetadata, InstallAction};

    fn make_meta(stars: u32, commits: u32, forks: u32) -> SkillMetadata {
        SkillMetadata {
            skill_id: "test".into(),
            name: "Test".into(),
            description: "Test skill description".into(),
            source_repo: "anthropics/skills".into(),
            source_path: "test/SKILL.md".into(),
            source_url: "https://github.com/anthropics/skills/tree/main/test".into(),
            tags: vec!["test".into()],
            stars,
            context_size: 1000,
            domain: "development".into(),
            last_updated: "2026-04-10".into(),
            install_action: InstallAction::Copy { path: "test/SKILL.md".into() },
        }
    }

    #[test]
    fn test_score_calculation() {
        let scorer = Scorer::default();
        let meta = make_meta(5000, 10, 100);
        let score = scorer.calculate(&meta);
        assert!(score > 0.0 && score <= 100.0);
    }

    #[test]
    fn test_high_score() {
        let scorer = Scorer::default();
        let meta = make_meta(10000, 20, 500);
        let score = scorer.calculate(&meta);
        assert!(score > 80.0);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test scorer_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/registry/scorer.rs`:

```rust
use crate::models::SkillMetadata;

pub struct Scorer {
    stars_weight: f32,
    activity_weight: f32,
    forks_weight: f32,
    stars_cap: u32,
    activity_target: u32,
    forks_cap: u32,
}

impl Default for Scorer {
    fn default() -> Self {
        Self {
            stars_weight: 0.50,
            activity_weight: 0.30,
            forks_weight: 0.20,
            stars_cap: 10000,
            activity_target: 10,
            forks_cap: 1000,
        }
    }
}

impl Scorer {
    pub fn calculate(&self, meta: &SkillMetadata) -> f32 {
        let stars_norm = (meta.stars as f32 / self.stars_cap as f32).min(1.0);
        let activity_norm = 1.0;
        let forks_norm = 0.5;
        
        let score = stars_norm * self.stars_weight +
                    activity_norm * self.activity_weight +
                    forks_norm * self.forks_weight;
        
        score * 100.0
    }

    pub fn filter_above_threshold(&self, skills: Vec<(SkillMetadata, f32)>, min: f32) -> Vec<(SkillMetadata, f32)> {
        skills.into_iter().filter(|(_, s)| *s >= min).collect()
    }

    pub fn sort_and_limit(&self, skills: Vec<(SkillMetadata, f32)>, limit: usize) -> Vec<(SkillMetadata, f32)> {
        let mut sorted = skills;
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        sorted.truncate(limit);
        sorted
    }
}
```

**Step 4: Update mod.rs**

Modify `src/registry/mod.rs`:

```rust
pub mod data;
pub mod fetch;
pub mod cache;
pub mod source;
pub mod github;
pub mod parser;
pub mod scorer;

#[cfg(test)]
mod data_tests;
mod source_tests;
mod github_tests;
mod parser_tests;
mod scorer_tests;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
pub use source::SourceRepo;
pub use github::GitHubClient;
pub use parser::{SkillParser, ParsedSkill};
pub use scorer::Scorer;
```

**Step 5: Run test**

Run: `cargo test scorer_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/registry/scorer.rs src/registry/mod.rs src/registry/scorer_tests.rs
git commit -m "feat: add Scorer for skill quality scoring"
```

---

## Task 10: Create Registry Generator

**Files:**
- Create: `src/registry/generator.rs`
- Modify: `src/registry/mod.rs`
- Create: `src/registry/generator_tests.rs`

**Step 1: Write failing test**

Create `src/registry/generator_tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::registry::RegistryGenerator;
    use crate::models::{Skill, InstallAction};
    use std::collections::HashMap;

    fn make_skill(id: &str) -> Skill {
        Skill {
            name: id.into(),
            description: format!("{} description", id),
            source_url: format!("https://github.com/test/skills/tree/main/{}", id),
            stars: 100,
            context_size: 500,
            domain: "test".into(),
            last_updated: "2026-04-10".into(),
            tags: vec!["test".into()],
            install_action: Some(InstallAction::Copy { path: format!("{}SKILL.md", id) }),
            score: Some(80.0),
        }
    }

    #[test]
    fn test_generate_registry() {
        let gen = RegistryGenerator::new(1);
        let mut skills = HashMap::new();
        skills.insert("tdd".into(), make_skill("tdd"));
        skills.insert("debug".into(), make_skill("debug"));
        
        let registry = gen.generate(skills);
        assert_eq!(registry.skills.len(), 2);
    }

    #[test]
    fn test_toml_output() {
        let gen = RegistryGenerator::new(1);
        let mut skills = HashMap::new();
        skills.insert("test".into(), make_skill("test"));
        
        let registry = gen.generate(skills);
        let toml = gen.to_toml(&registry);
        assert!(toml.contains("version = 1"));
        assert!(toml.contains("[skills.test]"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test generator_tests`
Expected: FAIL

**Step 3: Write implementation**

Create `src/registry/generator.rs`:

```rust
use crate::models::Registry;
use std::collections::HashMap;
use chrono::Utc;

pub struct RegistryGenerator {
    version: u32,
}

impl RegistryGenerator {
    pub fn new(version: u32) -> Self {
        Self { version }
    }

    pub fn generate(&self, skills: HashMap<String, crate::models::Skill>) -> Registry {
        Registry {
            version: self.version,
            updated: Utc::now().format("%Y-%m-%d").to_string(),
            skills,
        }
    }

    pub fn to_toml(&self, registry: &Registry) -> String {
        toml::to_string_pretty(registry).unwrap_or_default()
    }

    pub fn write(&self, registry: &Registry, path: &std::path::Path) -> crate::utils::Result<()> {
        std::fs::write(path, self.to_toml(registry))?;
        Ok(())
    }
}
```

**Step 4: Update mod.rs**

Modify `src/registry/mod.rs`:

```rust
pub mod data;
pub mod fetch;
pub mod cache;
pub mod source;
pub mod github;
pub mod parser;
pub mod scorer;
pub mod generator;

#[cfg(test)]
mod data_tests;
mod source_tests;
mod github_tests;
mod parser_tests;
mod scorer_tests;
mod generator_tests;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
pub use source::SourceRepo;
pub use github::GitHubClient;
pub use parser::{SkillParser, ParsedSkill};
pub use scorer::Scorer;
pub use generator::RegistryGenerator;
```

**Step 5: Run test**

Run: `cargo test generator_tests`
Expected: PASS

**Step 6: Commit**

```bash
git add src/registry/generator.rs src/registry/mod.rs src/registry/generator_tests.rs
git commit -m "feat: add RegistryGenerator for TOML output"
```

---

## Task 11: Create update-registry Binary

**Files:**
- Create: `src/bin/update-registry.rs`
- Modify: `Cargo.toml`

**Step 1: Add binary to Cargo.toml**

Add after `[dev-dependencies]`:

```toml
[[bin]]
name = "update-registry"
path = "src/bin/update-registry.rs"
```

**Step 2: Create directory**

Run: `mkdir -p src/bin`

**Step 3: Create binary**

Create `src/bin/update-registry.rs`:

```rust
use anyhow::Result;
use rulesify::{
    models::{SkillMetadata, InstallAction},
    registry::{GitHubClient, SourceRepo, SkillParser, Scorer, RegistryGenerator},
};
use std::collections::HashMap;

async fn fetch_skill(client: &GitHubClient, source: SourceRepo, path: &str, repo_stars: u32) -> Result<SkillMetadata> {
    let content = client.fetch_file(source.owner(), source.repo(), path).await?;
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
        install_action: InstallAction::Copy { path: path.into() },
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
        
        let tree = match client.fetch_tree(source.owner(), source.repo(), source.branch()).await {
            Ok(t) => t,
            Err(e) => {
                log::warn!("Failed to fetch tree: {}", e);
                continue;
            }
        };
        
        for entry in tree.tree.iter().filter(|e| source.matches_pattern(&e.path)) {
            if let Ok(meta) = fetch_skill(&client, source, &entry.path, repo.stargazers_count).await {
                let score = scorer.calculate(&meta);
                all_skills.push((meta, score));
                log::debug!("Skill {} scored {:.1}", meta.skill_id, score);
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
```

**Step 4: Verify build**

Run: `cargo check`
Expected: PASS

**Step 5: Commit**

```bash
git add src/bin/update-registry.rs Cargo.toml
git commit -m "feat: add update-registry binary"
```

---

## Task 12: Create GitHub Actions Workflow

**Files:**
- Create: `.github/workflows/update-registry.yml`

**Step 1: Create workflow**

Run: `mkdir -p .github/workflows`

Create `.github/workflows/update-registry.yml`:

```yaml
name: Update Registry

on:
  schedule:
    - cron: '0 6 * * 0'
  workflow_dispatch:

jobs:
  update:
    runs-on: ubuntu-latest
    env:
      GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
    
    steps:
      - uses: actions/checkout@v4
      
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      
      - run: cargo run --bin update-registry
      
      - id: changes
        run: |
          git diff --quiet registry.toml && echo "has_changes=false" >> $GITHUB_OUTPUT
          git diff --quiet registry.toml || echo "has_changes=true" >> $GITHUB_OUTPUT
      
      - if: steps.changes.outputs.has_changes == 'true'
        uses: peter-evans/create-pull-request@v6
        with:
          title: "Weekly registry update"
          branch: "auto/registry-update"
          commit-message: "chore: update registry"
```

**Step 2: Commit**

```bash
git add .github/workflows/update-registry.yml
git commit -m "feat: add GitHub Actions workflow for weekly updates"
```

---

## Task 13: Update Existing registry.toml

**Files:**
- Modify: `registry.toml`

**Step 1: Update to new format**

Modify `registry.toml`:

```toml
version = 1
updated = "2026-04-14"

[skills.test-driven-development]
name = "Test-Driven Development"
description = "Write tests before implementation code using TDD methodology"
source_url = "https://github.com/mattpocock/skills/tree/main/tdd"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["testing", "development", "best-practices"]
install_action = { type = "copy", value = "tdd/SKILL.md" }

[skills.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method before proposing fixes"
source_url = "https://github.com/anthropics/skills/tree/main/debugging"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["debugging", "troubleshooting"]
install_action = { type = "copy", value = "skills/debugging/SKILL.md" }

[skills.brainstorming]
name = "Brainstorming"
description = "Explore user intent and design before implementation"
source_url = "https://github.com/anthropics/skills/tree/main/brainstorming"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["design", "planning"]
install_action = { type = "copy", value = "skills/brainstorming/SKILL.md" }

[skills.verification-before-completion]
name = "Verification Before Completion"
description = "Run verification commands before claiming work is complete"
source_url = "https://github.com/anthropics/skills/tree/main/verification"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["quality", "verification"]
install_action = { type = "copy", value = "skills/verification/SKILL.md" }

[skills.writing-plans]
name = "Writing Plans"
description = "Create implementation plans before touching code"
source_url = "https://github.com/anthropics/skills/tree/main/planning"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["planning", "process"]
install_action = { type = "copy", value = "skills/planning/SKILL.md" }
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 3: Commit**

```bash
git add registry.toml
git commit -m "feat: update registry.toml to new format with install_action"
```

---

## Final Verification

Run: `cargo test && cargo clippy && cargo fmt`
Expected: All pass

---

**Summary:**

Created:
- `src/models/install_action.rs` - Copy/Command enum
- `src/models/skill_metadata.rs` - Raw skill data
- `src/registry/source.rs` - SourceRepo enum
- `src/registry/github.rs` - GitHub API client
- `src/registry/parser.rs` - SKILL.md parser
- `src/registry/scorer.rs` - Quality scoring
- `src/registry/generator.rs` - TOML generation
- `src/bin/update-registry.rs` - Automation binary
- `.github/workflows/update-registry.yml` - Weekly updates

Modified:
- `src/models/skill.rs` - Added stars, context_size, domain, install_action
- `src/models/registry.rs` - New skill format
- `registry.toml` - New format with install_action

---

**Plan complete. Execution options:**

**1. Subagent-Driven (this session)** - Task-by-task with review

**2. Parallel Session** - Batch execution in separate session

Which approach?