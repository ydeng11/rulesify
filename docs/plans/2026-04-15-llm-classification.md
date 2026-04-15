# LLM-Based Domain & Tag Classification Implementation

**Date**: 2026-04-15
**Status**: Completed

## Overview

Implemented LLM-based classification to derive domain (from 10 predefined) and tags (max 3) for each skill. Results cached in registry.toml to avoid re-evaluation.

## Predefined Domains (10)

```
planning-and-workflows
development
design-and-media
documentation
data-and-research
testing-and-debugging
deployment-and-infrastructure
integrations-and-tools
collaboration-and-communication
security-and-privacy
```

## Architecture

### New Modules

```
src/
├── models/
│   └── domain.rs          # Domain enum with 10 variants
│   └── domain_tests.rs    # Unit tests
├── llm/
│   ├── mod.rs             # Module exports
│   ├── client.rs          # OpenRouter HTTP client
│   ├── classifier.rs      # Batch classification logic
│   └── prompt.rs          # System/user prompt builder
```

### Modified Files

| File | Change |
|------|--------|
| `src/models/mod.rs` | Added `pub mod domain; pub use domain::Domain;` |
| `src/lib.rs` | Added `pub mod llm;` |
| `src/registry/source.rs` | Removed `domain()` method |
| `src/bin/update-registry.rs` | Added classification + `--force` flag |

## LLM Integration

### Configuration

| Env Var | Purpose | Default |
|---------|---------|---------|
| `OPENROUTER_API_KEY` | API authentication | Required |
| `OPENROUTER_MODEL` | Model selection | `anthropic/claude-3.5-haiku` |

### Batching

- **Batch size**: 25 skills per request
- **Processing**: Sequential batches
- **Input format**: `{ "skill-name": { "description": "..." } }`
- **Output format**: `{ "skill-name": { "domain": "...", "tags": [...] } }`

### System Prompt

```
You classify AI agent skills into domains and assign relevant tags.

Domains (choose one):
- planning-and-workflows
- development
- design-and-media
- documentation
- data-and-research
- testing-and-debugging
- deployment-and-infrastructure
- integrations-and-tools
- collaboration-and-communication
- security-and-privacy

Tags: Choose up to 3 tags describing the skill's capabilities. Tags should be lowercase, hyphenated if needed, and specific to the skill's purpose.

Input format: { "<skill>": { "description": "<words>" } }
Output format: { "<skill>": { "domain": "<domain>", "tags": ["<tag1>", "<tag2>"] } }

Classify each skill and respond ONLY with the JSON output format. Do not include any explanation or additional text.
```

## Cache Strategy

- **Location**: Embedded in registry.toml
- **Logic**: 
  1. Read existing registry.toml at start
  2. Build cache: skills with existing non-empty domain
  3. Skip LLM for cached skills
  4. Only classify new/unclassified skills
- **Override**: Use `--force` flag to re-classify all skills

## Error Handling

| Scenario | Behavior |
|----------|----------|
| LLM HTTP error | Retry once, then skip batch with fallback |
| Invalid domain in response | Fallback to `development` |
| Skill missing from response | Log warning, use fallback |
| Too many tags | Truncate to 3 |
| Rate limit (429) | Exponential backoff retry (max 3) |
| `OPENROUTER_API_KEY` missing | Exit with error |

## CLI Usage

```bash
# Normal update (uses cache)
GITHUB_TOKEN=xxx OPENROUTER_API_KEY=yyy cargo run --bin update-registry

# Force re-classification
cargo run --bin update-registry -- --force

# Custom model
OPENROUTER_MODEL=openai/gpt-4o-mini cargo run --bin update-registry

# Debug logging
RUST_LOG=debug cargo run --bin update-registry -- --force
```

## Test Results

All 35 tests passed:
- 10 domain tests (parsing, serialization, validation)
- Existing tests unaffected

## Key Implementation Details

1. **Domain enum**: `FromStr` for parsing LLM response, `Display` for serialization, serde rename_all="kebab-case"
2. **Client**: reqwest with 60s timeout, retry logic for rate limits
3. **Classifier**: Batch skills into 25-sized groups, parse JSON response with fallback handling
4. **Prompt builder**: Clean JSON response extraction (strip markdown code blocks if present)

## Verification

```bash
cargo check  # Pass
cargo test   # 35 tests pass
cargo clippy # 5 existing warnings (not from new code)
```