# Rebuild Design: Agents & Skills Auto-Loading

**Created:** 2026-04-08
**Status:** Approved

## Overview

Complete rewrite of rulesify from a Universal Rule File manager to an agent/skill discovery and recommendation tool. The new version helps AI agents self-configure by discovering and installing skills from a curated GitHub registry.

## Tech Stack

| Component | Choice | Rationale |
|-----------|--------|-----------|
| **CLI** | `clap` (derive) | Excellent ergonomics, already familiar |
| **Async Runtime** | `tokio` (rt-multi-thread) | Reqwest requires async |
| **HTTP Client** | `reqwest` | Well-maintained, integrates with tokio |
| **TUI** | `ratatui` + `crossterm` | Lightweight, cross-platform |
| **Serialization** | `serde` + `toml` | Human-readable registry format |
| **Error Handling** | `anyhow` + `thiserror` | Proven pattern |
| **Logging** | `env_logger` + `log` | Simple, works well |
| **Paths** | `dirs` | Home directory detection |
| **File Walking** | `walkdir` | Project scanning |

### Removed (Legacy)
- `serde_yaml`, `serde_json`, `chrono`, `glob`, `regex`

### New Dependencies
- `ratatui`, `crossterm`, `reqwest`, `toml`

### Cargo.toml (New)
```toml
[package]
name = "rulesify"
version = "0.4.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.37", features = ["rt-multi-thread", "macros"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
ratatui = "0.26"
crossterm = "0.27"
dirs = "5.0"
walkdir = "2.4"
env_logger = "0.10"
log = "0.4"

[dev-dependencies]
tempfile = "3.8"
```

## Architecture: Layered

```
src/
├── main.rs              # Entry point, CLI setup
├── cli/                 # Clap commands
│   ├── mod.rs
│   ├── init.rs          # Interactive setup flow
│   └── skill.rs         # Skill management commands
├── registry/            # Registry management
│   ├── mod.rs
│   ├── data.rs          # Built-in registry (include_str!)
│   ├── fetch.rs         # GitHub fetch logic
│   └── cache.rs         # Local cache management
├── scanner/             # Project context detection
│   ├── mod.rs
│   ├── language.rs      # Detect languages
│   ├── framework.rs     # Detect frameworks
│   └── tool_config.rs   # Detect AI tool configs
├── tui/                 # Terminal UI
│   ├── mod.rs
│   ├── skill_selector.rs
│   └── tool_picker.rs
├── installer/           # Generate instructions
│   ├── mod.rs
│   └── instructions.rs  # Tool-specific templates
├── models/              # Data structures
│   ├── mod.rs
│   ├── skill.rs
│   ├── registry.rs
│   └── context.rs
└── utils/
    ├── mod.rs
    └── error.rs
```

## Registry Structure (TOML)

```toml
# registry.toml
version = 1
updated = "2026-04-08"

[skills.test-driven-development]
name = "Test-Driven Development"
description = "Write tests before implementation code"
source = "https://github.com/anthropic/skills/tree/main/tdd"
tags = ["testing", "development", "best-practices"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]
popularity = 150

[skills.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method"
source = "https://github.com/anthropic/skills/tree/main/debugging"
tags = ["debugging", "troubleshooting"]
compatible_tools = ["cursor", "claude-code", "cline"]
popularity = 89
```

Built-in via `include_str!("../registry.toml")` with optional runtime fetch to update cache.

## Commands

```
rulesify init           # Interactive setup
rulesify skill list     # Show installed skills
rulesify skill add <id> # Add from registry
rulesify skill remove <id> # Remove skill
rulesify skill update   # Refresh cache
rulesify --help
rulesify --version
```

All legacy commands removed: `rule`, `deploy`, `import`, `validate`, `sync`, `config`, `completion`.

## Project Scanner

### Languages
- Scan file extensions: `.rs`, `.ts`, `.tsx`, `.js`, `.py`, `.go`, `.java`
- Parse config files: `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`

### Frameworks
- Extract from dependencies in package files
- Map common patterns (React, Next.js, Django, Tokio, etc.)

### AI Tool Configs
- `.cursor/rules/` → Cursor
- `CLAUDE.md` → Claude Code
- `.clinerules` → Cline
- `.goosehints` → Goose

Output: `ProjectContext { languages: Vec<String>, frameworks: Vec<String>, existing_tools: Vec<String> }`

## TUI Flow

1. **Tool Picker**: Multi-select which AI tools (cursor, claude-code, cline, goose)
2. **Context Display**: Show detected languages, frameworks, existing configs
3. **Skill Selector**: Browse/select skills filtered by project tags + selected tools
   - Columns: [✓] Name, Description, Stars, Tools
   - Arrow keys, Space, Enter
4. **Output**: Installation instructions for AI to execute

## Skill Tracking

`.rulesify.toml` in project root:
```toml
version = 1
tools = ["cursor", "claude-code"]

[installed_skills.test-driven-development]
added = "2026-04-08"
source = "https://github.com/anthropic/skills/tree/main/tdd"
```

## Data Flow

```
User runs: rulesify init
    ↓
Scanner detects project context
    ↓
TUI: Select AI tools (ToolPicker)
    ↓
Registry: Load built-in or fetch from GitHub
    ↓
Matcher: Filter skills by context + tools
    ↓
TUI: Select skills (SkillSelector)
    ↓
Installer: Generate instructions per tool
    ↓
Output: Print instructions for AI
    ↓
Save: .rulesify.toml tracking file
```

## Error Handling

- Registry fetch fails → Use built-in, show warning
- No skills match → Suggest broadening filters
- `.rulesify.toml` missing → Create on first `skill add`
- Invalid skill ID → List available skills

## Testing Strategy

- `scanner/`: Unit tests for each detector
- `registry/`: Mock HTTP responses for fetch tests
- `matcher/`: Test filtering logic
- `tui/`: Integration tests (harder, may skip)
- `installer/`: Template output verification

---
*Design approved: 2026-04-08*