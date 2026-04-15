# Contents API Refactor Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Refactor registry to use GitHub Contents API for targeted skill folder downloads, removing token requirement.

**Architecture:** Contents API lists folder contents, then fetches each file. Skills stored as folders (not single SKILL.md). Install action uses `folder` path. Graceful degradation on rate limits.

**Tech Stack:** Rust (tokio async, reqwest), GitHub Contents API v3

---

## Task 1: Update InstallAction Model

**Files:**
- Modify: `src/models/install_action.rs`
- Modify: `src/models/install_action_tests.rs`
- Modify: `registry.toml`

**Step 1: Write failing test**

Modify `src/models/install_action_tests.rs` to test folder field:

```rust
#[test]
fn test_copy_action_with_folder() {
    let action = InstallAction::Copy { folder: "debugging".to_string() };
    assert!(action.is_simple());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test install_action_tests::test_copy_action_with_folder`
Expected: FAIL (field `path` doesn't match)

**Step 3: Update InstallAction struct**

Modify `src/models/install_action.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum InstallAction {
    #[serde(rename = "copy")]
    Copy { folder: String },
    #[serde(rename = "command")]
    Command { value: String },
}

impl InstallAction {
    pub fn is_simple(&self) -> bool {
        matches!(self, InstallAction::Copy { .. })
    }

    pub fn install_command(&self, source_url: &str) -> Option<String> {
        match self {
            InstallAction::Copy { folder } => {
                Some(format!("rulesify skill fetch {} {}", source_url, folder))
            },
            InstallAction::Command { value } => Some(value.clone()),
        }
    }

    pub fn default_copy(skill_folder: &str) -> Self {
        InstallAction::Copy { folder: skill_folder.to_string() }
    }
}
```

**Step 4: Run tests to verify**

Run: `cargo test install_action_tests`
Expected: PASS

**Step 5: Update registry.toml**

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
install_action = { type = "copy", value = { folder = "tdd" } }

[skills.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method before proposing fixes"
source_url = "https://github.com/anthropics/skills/tree/main/skills/debugging"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["debugging", "troubleshooting"]
install_action = { type = "copy", value = { folder = "skills/debugging" } }

[skills.brainstorming]
name = "Brainstorming"
description = "Explore user intent and design before implementation"
source_url = "https://github.com/anthropics/skills/tree/main/skills/brainstorming"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["design", "planning"]
install_action = { type = "copy", value = { folder = "skills/brainstorming" } }

[skills.verification-before-completion]
name = "Verification Before Completion"
description = "Run verification commands before claiming work is complete"
source_url = "https://github.com/anthropics/skills/tree/main/skills/verification"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["quality", "verification"]
install_action = { type = "copy", value = { folder = "skills/verification" } }

[skills.writing-plans]
name = "Writing Plans"
description = "Create implementation plans before touching code"
source_url = "https://github.com/anthropics/skills/tree/main/skills/planning"
stars = 100
context_size = 0
domain = "development"
last_updated = "2026-04-14"
tags = ["planning", "process"]
install_action = { type = "copy", value = { folder = "skills/planning" } }
```

**Step 6: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 7: Commit**

```bash
git add src/models/install_action.rs src/models/install_action_tests.rs registry.toml
git commit -m "refactor: change install_action from path to folder"
```

---

## Task 2: Add Contents API Types

**Files:**
- Modify: `src/registry/github.rs`

**Step 1: Add new response types**

Modify `src/registry/github.rs` to add:

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct ContentEntry {
    pub name: String,
    pub path: String,
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(default)]
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileContent {
    pub content: Option<String>,
    pub encoding: Option<String>,
    #[serde(default)]
    pub download_url: Option<String>,
}
```

**Step 2: Run cargo check**

Run: `cargo check`
Expected: PASS

**Step 3: Commit**

```bash
git add src/registry/github.rs
git commit -m "feat: add ContentEntry and FileContent types for Contents API"
```

---

## Task 3: Add Contents API Methods

**Files:**
- Modify: `src/registry/github.rs`
- Modify: `src/registry/github_tests.rs`

**Step 1: Write failing test**

Add to `src/registry/github_tests.rs`:

```rust
#[test]
fn test_contents_api_url_format() {
    let client = GitHubClient::new(None);
    let url = client.contents_url("anthropics", "skills", "skills/debugging");
    assert_eq!(url, "https://api.github.com/repos/anthropics/skills/contents/skills/debugging");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test github_tests::test_contents_api_url_format`
Expected: FAIL

**Step 3: Add methods to GitHubClient**

Modify `src/registry/github.rs` to add methods:

```rust
impl GitHubClient {
    pub fn contents_url(&self, owner: &str, repo: &str, path: &str) -> String {
        format!("https://api.github.com/repos/{}/{}contents/{}", owner, repo, path)
    }

    pub async fn list_folder(&self, owner: &str, repo: &str, path: &str) -> Result<Vec<ContentEntry>> {
        let url = self.contents_url(owner, repo, path);
        let resp = self.request(&url)
            .send()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()))?;
        
        if !resp.status().is_success() {
            return Err(RulesifyError::GitHubApi(format!("HTTP {}", resp.status())).into());
        }
        
        resp.json::<Vec<ContentEntry>>()
            .await
            .map_err(|e| RulesifyError::GitHubApi(e.to_string()).into())
    }

    pub async fn fetch_file_raw(&self, owner: &str, repo: &str, path: &str) -> Result<String> {
        let url = format!(
            "https://raw.githubusercontent.com/{}/{}main/{}",
            owner, repo, path
        );
        
        let resp = self.http.get(&url)
            .header("User-Agent", "rulesify")
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

    pub async fn fetch_skill_folder(&self, owner: &str, repo: &str, folder_path: &str) -> Result<Vec<(String, String)>> {
        let mut files: Vec<(String, String)> = vec![];
        
        let entries = self.list_folder(owner, repo, folder_path).await?;
        
        for entry in entries {
            match entry.content_type.as_str() {
                "file" => {
                    let content = self.fetch_file_raw(owner, repo, &entry.path).await?;
                    files.push((entry.path, content));
                },
                "dir" => {
                    let subfolder_files = self.fetch_skill_folder(owner, repo, &entry.path).await?;
                    files.extend(subfolder_files);
                },
                _ => {}
            }
        }
        
        Ok(files)
    }
}
```

**Step 4: Run test to verify**

Run: `cargo test github_tests::test_contents_api_url_format`
Expected: PASS

**Step 5: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 6: Commit**

```bash
git add src/registry/github.rs src/registry/github_tests.rs
git commit -m "feat: add Contents API methods (list_folder, fetch_file_raw, fetch_skill_folder)"
```

---

## Task 4: Update SkillMetadata for Folder Path

**Files:**
- Modify: `src/models/skill_metadata.rs`
- Modify: `src/models/skill_metadata_tests.rs`

**Step 1: Update test**

Modify `src/models/skill_metadata_tests.rs`:

```rust
#[test]
fn test_metadata_creation() {
    let meta = SkillMetadata {
        skill_id: "tdd".to_string(),
        name: "Test-Driven Development".to_string(),
        description: "Write tests before implementation".to_string(),
        source_repo: "mattpocock/skills".to_string(),
        source_folder: "tdd".to_string(),
        source_url: "https://github.com/mattpocock/skills/tree/main/tdd".to_string(),
        tags: vec!["testing".to_string()],
        stars: 1500,
        context_size: 2400,
        domain: "development".to_string(),
        last_updated: "2026-04-10".to_string(),
        install_action: InstallAction::Copy { folder: "tdd".to_string() },
    };
    assert_eq!(meta.skill_id, "tdd");
    assert!(meta.install_action.is_simple());
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test skill_metadata_tests`
Expected: FAIL (field mismatch)

**Step 3: Update SkillMetadata struct**

Modify `src/models/skill_metadata.rs`:

```rust
use serde::{Deserialize, Serialize};
use crate::models::{Skill, InstallAction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub skill_id: String,
    pub name: String,
    pub description: String,
    pub source_repo: String,
    pub source_folder: String,
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

**Step 4: Run test to verify**

Run: `cargo test skill_metadata_tests`
Expected: PASS

**Step 5: Commit**

```bash
git add src/models/skill_metadata.rs src/models/skill_metadata_tests.rs
git commit -m "refactor: change SkillMetadata source_path to source_folder"
```

---

## Task 5: Update SourceRepo for Folder Discovery

**Files:**
- Modify: `src/registry/source.rs`
- Modify: `src/registry/source_tests.rs`

**Step 1: Update tests**

Modify `src/registry/source_tests.rs`:

```rust
#[test]
fn test_parse_skill_folder() {
    let anthropic = SourceRepo::AnthropicSkills;
    let folder = anthropic.parse_skill_folder("skills/debugging/SKILL.md");
    assert_eq!(folder, Some("skills/debugging".to_string()));
    
    let mattpocock = SourceRepo::MattPocockSkills;
    let folder = mattpocock.parse_skill_folder("tdd/SKILL.md");
    assert_eq!(folder, Some("tdd".to_string()));
    
    let openai = SourceRepo::OpenAISkillsCurated;
    let folder = openai.parse_skill_folder("skills/.curated/gh-fix-ci/SKILL.md");
    assert_eq!(folder, Some("skills/.curated/gh-fix-ci".to_string()));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test source_tests::test_parse_skill_folder`
Expected: FAIL

**Step 3: Add parse_skill_folder method**

Modify `src/registry/source.rs` to add:

```rust
pub fn parse_skill_folder(&self, path: &str) -> Option<String> {
    if !path.ends_with("SKILL.md") {
        return None;
    }
    
    let folder_path = path.replace("/SKILL.md", "");
    Some(folder_path)
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
```

**Step 4: Run test to verify**

Run: `cargo test source_tests::test_parse_skill_folder`
Expected: PASS

**Step 5: Commit**

```bash
git add src/registry/source.rs src/registry/source_tests.rs
git commit -m "feat: add parse_skill_folder method to SourceRepo"
```

---

## Task 6: Update update-registry Binary

**Files:**
- Modify: `src/bin/update-registry.rs`

**Step 1: Update fetch_skill function**

Modify `src/bin/update-registry.rs`:

```rust
async fn fetch_skill(
    client: &GitHubClient, 
    source: SourceRepo, 
    folder_path: &str, 
    repo_stars: u32
) -> Result<Option<SkillMetadata>> {
    let skill_id = source.parse_skill_id(&format!("{}SKILL.md", folder_path)).unwrap_or("unknown".into());
    let skill_md_path = format!("{}SKILL.md", folder_path);
    
    let skill_md = match client.fetch_file_raw(source.owner(), source.repo(), &skill_md_path).await {
        Ok(content) => content,
        Err(e) => {
            log::warn!("Failed to fetch {} SKILL.md: {}", skill_id, e);
            return Ok(None);
        }
    };
    
    let parsed = match SkillParser::parse(&skill_md) {
        Ok(p) => p,
        Err(e) => {
            log::warn!("Failed to parse {} frontmatter: {}", skill_id, e);
            return Ok(None);
        }
    };
    
    let context_size = SkillParser::estimate_context_size(&skill_md);
    
    let source_url = format!(
        "https://github.com/{}/skills/tree/{}/{}",
        source.owner(),
        source.branch(),
        folder_path
    );
    
    Ok(Some(SkillMetadata {
        skill_id,
        name: parsed.name,
        description: parsed.description,
        source_repo: source.full_name(),
        source_folder: folder_path.to_string(),
        source_url,
        tags: parsed.tags,
        stars: repo_stars,
        context_size,
        domain: source.domain().into(),
        last_updated: chrono::Utc::now().format("%Y-%m-%d").to_string(),
        install_action: InstallAction::Copy { folder: folder_path.to_string() },
    }))
}
```

**Step 2: Update main function discovery loop**

Modify `src/bin/update-registry.rs` main function:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let client = GitHubClient::new(None);
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
        
        for entry in tree.tree.iter().filter(|e| e.path.ends_with("SKILL.md")) {
            if let Some(folder_path) = source.parse_skill_folder(&entry.path) {
                if let Some(meta) = fetch_skill(&client, source, &folder_path, repo.stargazers_count).await? {
                    let score = scorer.calculate(&meta);
                    all_skills.push((meta, score));
                    log::debug!("Skill {} scored {:.1}", meta.skill_id, score);
                }
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

**Step 3: Run cargo check**

Run: `cargo check`
Expected: PASS

**Step 4: Commit**

```bash
git add src/bin/update-registry.rs
git commit -m "refactor: update binary to use folder paths instead of file paths"
```

---

## Task 7: Remove Token Requirement

**Files:**
- Modify: `src/bin/update-registry.rs`
- Modify: `src/registry/github.rs`

**Step 1: Update GitHubClient to be optional token**

Modify `src/registry/github.rs`:

```rust
impl GitHubClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("rulesify")
            .build()
            .unwrap();
        
        Self { http, token: None }
    }

    pub fn with_token(token: String) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("rulesify")
            .build()
            .unwrap();
        
        Self { http, token: Some(token) }
    }
}
```

**Step 2: Update binary**

Modify `src/bin/update-registry.rs`:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let client = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        log::info!("Using authenticated requests");
        GitHubClient::with_token(token)
    } else {
        log::info!("Using unauthenticated requests (60/hr rate limit)");
        GitHubClient::new()
    };
    
    // ... rest unchanged
}
```

**Step 3: Run cargo check**

Run: `cargo check`
Expected: PASS

**Step 4: Commit**

```bash
git add src/registry/github.rs src/bin/update-registry.rs
git commit -m "feat: make GitHub token optional with graceful rate limit handling"
```

---

## Task 8: Final Verification

**Step 1: Run all tests**

Run: `cargo test`
Expected: PASS

**Step 2: Run clippy**

Run: `cargo clippy`
Expected: No errors (warnings ok)

**Step 3: Format code**

Run: `cargo fmt`

**Step 4: Test binary runs**

Run: `cargo run --bin update-registry 2>&1 | head -10`
Expected: Starts fetching from repos (may fail due to rate limit, that's ok)

---

## Summary

**Changed files:**
- `src/models/install_action.rs` - `path` → `folder`
- `src/models/skill_metadata.rs` - `source_path` → `source_folder`
- `src/registry/github.rs` - Added Contents API methods
- `src/registry/source.rs` - Added `parse_skill_folder`
- `src/bin/update-registry.rs` - Uses folder discovery
- `registry.toml` - Updated install_action format

**API changes:**
- Uses Contents API (list_folder, fetch_file_raw)
- Uses raw.githubusercontent.com for file downloads (no auth needed)
- Uses Tree API only for discovery (optional auth)

**Rate limits:**
- Unauthenticated: 60 tree calls/hr (discovery)
- Raw URLs: unlimited (file downloads)
- Authenticated: 5000 calls/hr with token

---

**Plan complete. Two execution options:**

**1. Subagent-Driven (this session)** - Task-by-task with review

**2. Parallel Session** - Batch execution in separate session

Which approach?