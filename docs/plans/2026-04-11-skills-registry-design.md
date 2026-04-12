# Skills Registry Design - Hand-off Document

> **Status:** Approved, ready for implementation  
> **Date:** 2026-04-11  
> **Author:** Design session with user  
> **Next Phase:** Implementation planning → execution

---

## Executive Summary

**Goal:** Replace the current placeholder registry.toml with a real, sustainable skills registry that:
- Curates 20 core skills (manually verified)
- Auto-selects top 50 skills weekly (score-based)
- Updates automatically via GitHub Actions
- Uses configurable scoring formula for future extensibility

**Key Decisions:**
- Scoring formula: Stars (50%) + Activity (30%) + Forks (20%)
- Update frequency: Weekly (Sunday cron)
- Sources: anthropics/skills, mattpocock/skills, openai/skills
- Format: TOML with curated + selected sections
- Future LLM evaluation architecture designed but not implemented yet

---

## Problem Context

**Current State:**
- registry.toml has 5 placeholder skills with fake URLs
- No automation, no scoring, no curation methodology
- Skills are manually invented (not from real repos)

**User Requirements:**
1. Curate top 20-50 from all sources (not exhaustive catalog)
2. Prioritize: Development, Planning, Documents, Design categories
3. Include tool-specific skills with clear marking
4. Sustainable maintenance (automated updates)
5. Weekly refresh frequency

**Constraints:**
- GitHub API rate limits (60/hr unauthenticated, 5000/hr with token)
- Skills don't have individual stars (use repo stars as proxy)
- Manual verification is unsustainable at scale
- Need to handle different repo structures

---

## Architecture Overview

### Components

```
rulesify/
├── registry.toml                    # Generated output (TOML format)
├── scoring-config.yaml              # Configurable weights/thresholds
├── curated-skills.yaml              # Manual curated core (20 skills)
├── scripts/
│   ├── update-registry.rs           # Main automation script
│   ├── fetcher.rs                   # GitHub API fetching
│   ├── scorer.rs                    # Score calculation
│   ├── validator.rs                 # Structure validation
│   ├── generator.rs                 # TOML generation
│   └── llm-evaluate.rs              # Future LML evaluation (stub)
├── .github/
│   └── workflows/
│       └── update-registry.yml      # Weekly automation
└── src/
    └ registry/
        ├── fetcher.rs               # Repo-specific fetchers
        ├── scorer.rs                # Scoring module
        ├── validator.rs             # Structure checks
        └ generator.rs              # TOML generator
        └── cache.rs                 # API response cache
```

### Data Flow

```
[GitHub API] → Fetcher → [Raw Skill Metadata]
                           ↓
[SKILL.md files] → Validator → [Parsed Skills]
                           ↓
[scoring-config.yaml] → Scorer → [Scored Skills]
                           ↓
[curated-skills.yaml] + [Scored Skills] → Generator → [registry.toml]
                           ↓
[GitHub Actions] → Weekly Cron → [Updated registry.toml]
```

---

## Scoring System

### Formula (Configurable)

```yaml
# scoring-config.yaml
formula_version: 1
weights:
  stars: 0.50              # popularity signal
  recent_activity: 0.30    # maintenance signal
  forks: 0.20              # community engagement

# Future weights (disabled now)
# llm_simplicity: 0.0
# documentation_quality: 0.0

thresholds:
  min_score: 60            # exclude below this
  core_threshold: 90       # candidate for curated
  max_selected: 50         # limit auto-selected count

normalization:
  stars_cap: 10000         # max stars for normalization
  forks_cap: 1000          # max forks for normalization
  activity_target: 10      # commits in 30 days for max score
```

### Score Calculation (Rust)

```rust
fn calculate_score(skill: &SkillMetadata, config: &ScoringConfig) -> f32 {
    let stars_norm = (skill.repo_stars as f32 / config.normalization.stars_cap).min(1.0);
    let activity = (skill.repo_commits_recent as f32 / config.normalization.activity_target).min(1.0);
    let forks_norm = (skill.repo_forks as f32 / config.normalization.forks_cap).min(1.0);
    
    let base_score = 
        stars_norm * config.weights.stars +
        activity * config.weights.recent_activity +
        forks_norm * config.weights.forks;
    
    // Apply repo reputation modifier
    let reputation = get_repo_reputation(&skill.source_repo);
    
    base_score * reputation
}

fn get_repo_reputation(repo: &str) -> f32 {
    match repo {
        "anthropics/skills" => 1.0,
        "openai/skills" => 1.0,
        "mattpocock/skills" => 0.95,
        "openclaw/skills" => 0.70,
        _ => 0.50,  // unknown repos
    }
}
```

**Note:** Skills don't have individual stars. We use repo stars as proxy with reputation modifier to differentiate quality.

---

## Fetching Strategy

### Source Repos

| Repo | Structure | Skill Path Pattern | Reputation |
|------|-----------|-------------------|------------|
| anthropics/skills | `skills/{name}/SKILL.md` | Flat in skills/ dir | 1.0 |
| mattpocock/skills | `{name}/SKILL.md` | Flat in root | 0.95 |
| openai/skills | `skills/.curated/{name}/SKILL.md` | Nested categories | 1.0 |
| openai/skills | `skills/.system/{name}/SKILL.md` | System skills | 1.0 |
| openai/skills | `skills/.experimental/{name}/SKILL.md` | Experimental | 0.90 |
| openclaw/skills | `skills/{author}/{name}/SKILL.md` | Two-level hierarchy | 0.70 |

### Repo-Specific Fetchers

```rust
trait RepoFetcher {
    fn source_url(&self) -> &str;
    fn reputation(&self) -> f32;
    async fn discover_skills(&self, client: &GitHubClient) -> Result<Vec<SkillPath>>;
    async fn fetch_skill_metadata(&self, client: &GitHubClient, path: &SkillPath) -> Result<SkillMetadata>;
}

// Implementations for each repo type
struct AnthropicFetcher;
struct MattPocockFetcher;
struct OpenAIFetcher;  // handles .curated, .system, .experimental
struct OpenClawFetcher;
```

### Fetch Sequence

```
1. Initialize GitHubClient (token from env GITHUB_TOKEN)
2. Load scoring-config.yaml
3. For each source repo:
   a. GET /repos/{owner}/{repo} → repo metadata (stars, forks, pushed_at)
   b. GET /repos/{owner}/{repo}/git/trees/main?recursive=1 → directory tree
   c. Parse tree → find directories with SKILL.md files
   d. GET /repos/{owner}/{repo}/commits?since=30days → recent commits
   e. Calculate repo activity score
   f. For each skill discovered:
      - Check cache for SKILL.md
      - If not cached: GET /repos/{owner}/{repo}/contents/{path}/SKILL.md
      - Parse YAML frontmatter (name, description, tags)
      - Validate structure
      - Store SkillMetadata
4. Calculate scores for all skills
5. Filter by min_score threshold
6. Sort by score descending
7. Take top 50 for auto-selected
8. Load curated-skills.yaml (20 manual skills)
9. Merge curated + selected
10. Generate registry.toml
```

### GitHub API Calls Estimate

| Call Type | Count | Rate Limit Impact |
|-----------|-------|-------------------|
| Repo metadata | 3 | 3 calls |
| Directory trees | 6 | 6 calls |
| Recent commits | 3 | 3 calls |
| SKILL.md files | ~50-80 | 50-80 calls |
| **Total** | ~62-92 | Within 5000/hr limit with token |

### Rate Limit Handling

```rust
struct GitHubClient {
    token: Option<String>,
    rate_limiter: RateLimiter,
    cache: CacheStore,
}

impl GitHubClient {
    async fn fetch_with_rate_limit(&self, url: &str) -> Result<Response> {
        // Check cache first (24hr TTL)
        if let Some(cached) = self.cache.get(url) {
            return Ok(cached);
        }
        
        // Wait if approaching rate limit
        self.rate_limiter.wait_if_needed();
        
        // Make authenticated request
        let response = self.http_client.get(url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("If-None-Match", cached_etag)  // conditional request
            .send()?;
        
        // Cache with ETag for validation
        self.cache.set(url, response.clone());
        
        Ok(response)
    }
}
```

**Cache Strategy:**
- 24-hour TTL for all responses
- ETag-based cache validation (conditional requests)
- Reduces subsequent runs to ~10-20 calls (only changed files)

### Error Handling

| Error | Response |
|-------|----------|
| Repo 404 | Log warning, skip repo, continue |
| Skill missing SKILL.md | Skip skill, log warning |
| Rate limit exceeded | Wait, retry, or use cache |
| Invalid SKILL.md format | Exclude skill, log error |
| Network timeout | Retry 3×, then use cache |
| Parse failure | Exclude skill, continue |

**Key principle:** Never fail entire run. Skip problematic skills, continue with rest.

---

## Registry Output Format

### registry.toml Structure

```toml
version = 1
updated = "2026-04-11T00:00:00Z"
formula_version = 1

[meta]
sources_count = 3
total_skills = 72
curated_count = 20
selected_count = 50
excluded_count = 15  # failed min_score threshold

# Tier 1: Curated Core (manually verified, always present)
[curated.tdd]
name = "Test-Driven Development"
description = "Write tests before implementation using TDD methodology"
source = "https://github.com/mattpocock/skills/tree/main/tdd"
tags = ["testing", "development", "best-practices"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]

[curated.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method before proposing fixes"
source = "https://github.com/anthropics/skills/tree/main/skills/debugging"
tags = ["debugging", "troubleshooting", "investigation"]
compatible_tools = ["cursor", "claude-code", "cline"]

# Tier 2: Auto-Selected (score-based, weekly refresh)
[selected.gh-fix-ci]
name = "Fix GitHub CI"
description = "Automatically diagnose and fix failing GitHub CI workflows"
source = "https://github.com/openai/skills/tree/main/skills/.curated/gh-fix-ci"
tags = ["ci", "github", "automation"]
score = 78.5
compatible_tools = ["cursor", "claude-code"]
auto_selected_at = "2026-04-11T00:00:00Z"

[selected.notion-meeting-intelligence]
name = "Notion Meeting Intelligence"
description = "Extract action items and insights from meeting notes in Notion"
source = "https://github.com/openai/skills/tree/main/skills/.curated/notion-meeting-intelligence"
tags = ["notion", "meetings", "productivity"]
score = 72.3
compatible_tools = ["claude-code", "cursor"]
requires_tool = "notion"  # tool-specific marker
auto_selected_at = "2026-04-11T00:00:00Z"
```

### curated-skills.yaml (Manual Override)

```yaml
# curated-skills.yaml
# Hand-picked skills that always appear in registry
# Automation respects this file (won't overwrite curated section)

[curated.tdd]
name = "Test-Driven Development"
description = "Write tests before implementation using TDD methodology"
source = "https://github.com/mattpocock/skills/tree/main/tdd"
tags = ["testing", "development", "best-practices"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]
verified_at = "2026-04-11"
verified_by = "manual"

[curated.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method before proposing fixes"
source = "https://github.com/anthropics/skills/tree/main/skills/debugging"
tags = ["debugging", "troubleshooting"]
compatible_tools = ["cursor", "claude-code", "cline"]
verified_at = "2026-04-11"
verified_by = "manual"

# ... 18 more curated skills
```

**Separation rationale:**
- curated-skills.yaml = manual, stable, rarely changes
- registry.toml = generated output, weekly updates
- Automation merges curated + selected into registry.toml

---

## Weekly Automation Workflow

### GitHub Actions Workflow

```yaml
# .github/workflows/update-registry.yml
name: Update Skills Registry
on:
  schedule:
    - cron: '0 6 * * 0'  # Sunday 6am UTC
  workflow_dispatch:      # Manual trigger

env:
  GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout repo
        uses: actions/checkout@v4
        
      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        
      - name: Cache cargo dependencies
        uses: actions/cache@v4
        with:
          path: ~/.cargo
          key: cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Cache GitHub API responses
        uses: actions/cache@v4
        with:
          path: ~/.cache/rulesify-api
          key: api-cache-${{ github.run_id }}
          restore-keys: api-cache-
          
      - name: Run registry update script
        run: cargo run --bin update-registry
        
      - name: Check for changes
        id: changes
        run: |
          git diff --quiet registry.toml && echo "has_changes=false" >> $GITHUB_OUTPUT
          git diff --quiet registry.toml || echo "has_changes=true" >> $GITHUB_OUTPUT
          
      - name: Create PR if changes detected
        if: steps.changes.outputs.has_changes == 'true'
        uses: peter-evans/create-pull-request@v6
        with:
          title: "Weekly registry update"
          body: "Automated weekly skills registry refresh"
          branch: "auto/registry-update"
          commit-message: "chore: update skills registry (weekly)"
          
      - name: Auto-merge if only registry changes
        if: steps.changes.outputs.has_changes == 'true'
        run: |
          # Check if only registry.toml changed
          changed_files=$(git diff --name-only HEAD~1)
          if [[ "$changed_files" == "registry.toml" ]]; then
            gh pr merge --auto --squash
          fi
```

### Failure Handling

- Script fails → workflow fails → no registry update
- Previous registry.toml remains (fallback)
- Maintainer notified via GitHub Actions failure email
- Cache persists across runs (subsequent runs faster)

### Manual Override Process

1. Maintainer edits curated-skills.yaml manually
2. Pushes to main branch
3. Next weekly run respects curated (won't overwrite)
4. Changes appear in registry.toml after next run

---

## Curated Core (20 Skills)

### Development Process (8 skills)

| Skill | Source | Description |
|-------|--------|-------------|
| tdd | mattpocock/skills | TDD methodology, red-green-refactor |
| systematic-debugging | anthropics/skills | Scientific debugging method |
| triage-issue | mattpocock/skills | Bug investigation → GitHub issue |
| improve-codebase-architecture | mattpocock/skills | Architecture improvement analysis |
| webapp-testing | anthropics/skills | Web app testing workflows |
| gh-fix-ci | openai/skills | Diagnose and fix CI failures |
| request-refactor-plan | mattpocock/skills | Refactor planning via interview |
| brainstorming | (existing) | Design exploration before implementation |

### Planning & Design (5 skills)

| Skill | Source | Description |
|-------|--------|-------------|
| write-a-prd | mattpocock/skills | PRD via interactive interview |
| prd-to-plan | mattpocock/skills | PRD → implementation plan |
| prd-to-issues | mattpocock/skills | PRD → GitHub issues |
| grill-me | mattpocock/skills | Interview plan/design exhaustively |
| design-an-interface | mattpocock/skills | Generate interface designs |

### Documents & Content (4 skills)

| Skill | Source | Description |
|-------|--------|-------------|
| docx | anthropics/skills | Word document creation/editing |
| pdf | anthropics/skills | PDF manipulation |
| pptx | anthropics/skills | PowerPoint creation |
| xlsx | anthropics/skills | Excel spreadsheet handling |

### Design & UI (3 skills)

| Skill | Source | Description |
|-------|--------|-------------|
| frontend-design | anthropics/skills | Frontend design patterns |
| figma-implement-design | openai/skills | Figma → code implementation |
| canvas-design | anthropics/skills | Canvas-based design workflows |

### Selection Criteria

Each curated skill meets:
- ✅ Has SKILL.md with proper frontmatter
- ✅ Clear, actionable description
- ✅ Works with multiple tools OR clearly marked tool-specific
- ✅ From reputable source (anthropic, openai, mattpocock)
- ✅ Proven utility (stars/usage signals)

---

## Future: LLM Evaluation (Architecture)

### Not Implemented Yet, Designed for Future

```yaml
# scoring-config.yaml addition (future)
llm_evaluation:
  enabled: false            # toggle when ready
  model: "claude-3-haiku"   # cheap batch model
  weight: 0.25              # future weight in formula
  batch_size: 10            # process in batches
  prompts:
    simplicity: |
      Rate this skill's instructions for simplicity (1-100).
      Consider: clarity, actionability, brevity.
      Skill content:
      {skill_content}
```

### Implementation Stub

```rust
// scripts/llm-evaluate.rs (stub for future)

enum EvaluationCriteria {
    Simplicity,    // Can agent follow without confusion?
    Clarity,       // Are instructions understandable?
    Actionability, // Specific enough to execute?
    Brevity,       // No unnecessary verbosity?
}

struct LLMEvaluator {
    model: String,
    api_key: String,
}

impl LLMEvaluator {
    async fn evaluate_batch(&self, skills: &[SkillMetadata]) -> Vec<u8> {
        // Future: call LLM API with batch of skills
        // Return scores 0-100 for each skill
        unimplemented!("LLM evaluation not yet implemented")
    }
}

// Integration point in scorer.rs
fn calculate_score(&self, skill: &Skill, config: &Config) -> f32 {
    let base_score = self.base_score(skill, config);
    
    if config.llm_evaluation.enabled {
        let llm_score = self.llm_cache.get(&skill.source)
            .unwrap_or_else(|| self.llm_evaluator.evaluate(&skill));
        
        base_score * (1.0 - config.weights.llm) + 
        llm_score * config.weights.llm
    } else {
        base_score
    }
}
```

### When to Implement

- Phase 2 (after registry working)
- Budget available (~$5/month for 50 skills @ $0.001/skill)
- Use cheap model (haiku, gpt-4o-mini)
- Cache LLM scores (don't re-evaluate unchanged skills)

### LLM Cache Strategy

```rust
struct LLMCache {
    scores: HashMap<String, CachedScore>,
    ttl: Duration,  // 30 days
}

struct CachedScore {
    score: u8,
    evaluated_at: DateTime,
    skill_hash: String,  // hash of SKILL.md content
}

// Only re-evaluate if content changed
fn should_revaluate(&self, skill: &Skill) -> bool {
    let cached = self.cache.get(&skill.source)?;
    let current_hash = hash_content(&skill.content);
    
    cached.skill_hash != current_hash || cached.expired()
}
```

---

## Implementation Tasks

### Phase 1: Core Implementation

**Files to create:**
1. `scoring-config.yaml` - configurable weights
2. `curated-skills.yaml` - manual curated core (20 skills)
3. `scripts/update-registry.rs` - main automation binary
4. `src/registry/fetcher.rs` - GitHub API client + repo fetchers
5. `src/registry/scorer.rs` - score calculation module
6. `src/registry/validator.rs` - structure validation
7. `src/registry/generator.rs` - TOML generation
8. `src/registry/cache.rs` - API response cache
9. `.github/workflows/update-registry.yml` - weekly automation

**Dependencies to add:**
```toml
# Cargo.toml
[dependencies]
serde_yaml = "0.9"        # YAML config parsing
sha2 = "0.10"             # content hashing for cache
chrono = { version = "0.4", features = ["serde"] }  # timestamps
```

**Key functions:**
- `fetch_repo_metadata()` - GitHub API repo info
- `discover_skills()` - parse directory tree
- `fetch_skill_md()` - fetch SKILL.md content
- `parse_frontmatter()` - YAML extraction
- `validate_structure()` - required fields check
- `calculate_score()` - apply formula
- `generate_toml()` - output registry

### Phase 2: Testing

**Test cases needed:**
1. Unit: score calculation with various inputs
2. Unit: SKILL.md parsing (valid/invalid)
3. Unit: cache hit/miss behavior
4. Integration: full fetch sequence (mock GitHub API)
5. Integration: registry generation from mock data
6. E2E: actual GitHub API call (requires token)

### Phase 3: Deployment

1. Add `GITHUB_TOKEN` secret to repo
2. Enable GitHub Actions workflow
3. Run manual trigger to test
4. Verify first registry.toml output
5. Enable weekly cron schedule

### Phase 4: Documentation

1. Update AGENTS.md with registry architecture
2. Document scoring-config.yaml format
3. Document curated-skills.yaml maintenance
4. Add developer guide for LLM evaluation (future)

---

## Acceptance Criteria

### Must Have (Phase 1)

- [ ] scoring-config.yaml with configurable weights
- [ ] curated-skills.yaml with 20 verified skills
- [ ] Rust fetcher module connects to GitHub API
- [ ] Score calculation working (stars + activity + forks)
- [ ] Structure validation (SKILL.md frontmatter check)
- [ ] registry.toml generated with curated + selected sections
- [ ] Cache system reduces API calls on subsequent runs
- [ ] Error handling: skip failures, continue execution
- [ ] GitHub Actions workflow runs weekly
- [ ] PR created on changes, auto-merge if only registry
- [ ] All tests pass

### Should Have (Phase 2)

- [ ] LLM evaluation stub architecture ready
- [ ] Configurable repo reputation modifiers
- [ ] Cache persists across workflow runs
- [ ] Detailed logging for debugging

### Nice to Have (Phase 3+)

- [ ] LLM simplicity evaluation implemented
- [ ] Community contribution workflow
- [ ] Dynamic skill fetching at runtime (vs static registry)
- [ ] Skill versioning/tracking

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| GitHub API changes | Fetch fails | Pin API version, add fallback |
| Rate limit exceeded | Can't fetch | Token + cache + graceful degradation |
| Repo disappears | Skills invalid | Cache previous, log warning |
| SKILL.md format diverges | Parse fails | Flexible parser, validation warnings |
| Low-quality skills auto-selected | Bad registry | High min_score threshold, curated core anchor |
| LLM evaluation cost | Budget overrun | Only enable when budget approved, batch processing |

---

## Open Questions

1. **Should we include openclaw/skills?** Currently set reputation=0.70 (lower trust). User approved, but may want to exclude entirely.

2. **How to handle skill name conflicts?** Multiple repos may have skill with same name (e.g., "tdd" in multiple repos). Proposed: use `{repo}-{skill}` naming or prefer curated.

3. **Should compatible_tools be required?** Currently optional. Some skills don't specify tool compatibility. Proposed: if missing, assume universal (all tools).

4. **How to track skill changes?** Skills may change content. Proposed: hash SKILL.md content, detect changes, re-evaluate.

5. **Should we version skills?** Currently fetch latest (no version locking). Proposed: future feature if needed.

---

## Next Steps

1. **User approves this hand-off doc** → proceed to implementation planning
2. **Create implementation plan** → use writing-plans skill
3. **Execute implementation** → atomic commits, testing
4. **First manual registry generation** → verify output
5. **Deploy GitHub Actions workflow** → enable weekly cron
6. **Monitor first automated run** → adjust scoring if needed

---

## Quick Reference

### Key Files

```
registry.toml               # Output (generated)
scoring-config.yaml         # Config (editable)
curated-skills.yaml         # Manual overrides (editable)
scripts/update-registry.rs  # Automation (executable)
.github/workflows/update-registry.yml  # Weekly trigger
```

### Key Config Values

```yaml
weights:
  stars: 0.50
  recent_activity: 0.30
  forks: 0.20

thresholds:
  min_score: 60
  max_selected: 50

sources:
  - anthropics/skills (reputation: 1.0)
  - openai/skills (reputation: 1.0)
  - mattpocock/skills (reputation: 0.95)
```

### Key Commands

```bash
# Manual registry update
cargo run --bin update-registry

# Test fetch
cargo test --test fetch_integration

# View generated registry
cat registry.toml

# GitHub Actions manual trigger
gh workflow run update-registry.yml
```

---

## Appendix: SKILL.md Parsing

### Expected Format

```markdown
---
name: skill-name
description: Clear description of what this skill does
tags: ["tag1", "tag2"]
compatible_tools: ["cursor", "claude-code"]
---

# Skill Name

Instructions for the skill...

## Examples
- Example 1
- Example 2

## Guidelines
- Guideline 1
- Guideline 2
```

### Parser Implementation

```rust
fn parse_skill_md(content: &str) -> Result<ParsedSkill> {
    // Extract YAML frontmatter (between --- markers)
    let frontmatter = extract_frontmatter(content)?;
    
    // Parse YAML
    let metadata: SkillFrontmatter = serde_yaml::from_str(&frontmatter)?;
    
    // Validate required fields
    if metadata.name.is_empty() || metadata.description.len() < 20 {
        return Err(ParseError::InvalidFrontmatter);
    }
    
    // Extract markdown body (after frontmatter)
    let body = extract_body(content);
    
    Ok(ParsedSkill {
        frontmatter: metadata,
        body: body,
    })
}
```

### Validation Rules

- `name` required, non-empty
- `description` required, > 20 chars
- `tags` optional, default empty array
- `compatible_tools` optional, default assume universal
- Frontmatter must be valid YAML
- Content after frontmatter is body (not validated strictly)

---

## Appendix: Cache Data Structures

```rust
struct CacheStore {
    entries: HashMap<String, CacheEntry>,
    path: PathBuf,  // ~/.cache/rulesify-api/
    ttl: Duration,  // 24 hours
}

struct CacheEntry {
    url: String,
    response_body: String,
    etag: String,
    fetched_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
}

impl CacheStore {
    fn get(&self, url: &str) -> Option<&CacheEntry> {
        let entry = self.entries.get(url)?;
        if entry.expires_at < Utc::now() {
            None  // expired
        } else {
            Some(entry)
        }
    }
    
    fn set(&mut self, url: &str, response: Response) {
        let etag = response.headers().get("ETag").unwrap();
        self.entries.insert(url, CacheEntry {
            url: url.to_string(),
            response_body: response.text(),
            etag: etag.to_string(),
            fetched_at: Utc::now(),
            expires_at: Utc::now() + self.ttl,
        });
    }
}
```

**Cache location:** `~/.cache/rulesify-api/cache.json`

**Persistence:** Load on startup, save on shutdown, survive across runs.

---

**End of hand-off document. Ready for implementation planning phase.**