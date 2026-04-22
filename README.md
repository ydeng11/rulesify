# Rulesify

Rulesify is a **skill management tool for AI agents**. Discover, browse, and install pre-built skills from curated GitHub repositories to enhance your AI assistant's capabilities.

## What are Skills?

Skills are pre-built workflows, methodologies, and instructions that enhance AI agent capabilities. Each skill is a self-contained module that guides AI assistants through complex tasks like:

- **Test-driven development** - Write tests before implementation
- **Systematic debugging** - Investigate bugs with scientific method
- **Deployment workflows** - Deploy to Netlify, Render, Cloudflare
- **Document creation** - Generate PDFs, Word docs, presentations
- **Security reviews** - Threat modeling, security best practices
- **Design workflows** - Figma integration, UI generation

## Features

- **Skills Registry**: Browse and install 50+ skills from trusted sources
- **LLM Classification**: Automatic domain and tag assignment for easy discovery
- **Domain Filtering**: Find skills by category (development, testing, design, etc.)
- **Tag Search**: Discover skills by capability tags
- **Direct Installation**: Skills are downloaded and installed automatically (full folder copy)
- **Multi-Tool Support**: Install to multiple AI tools simultaneously (Claude Code, Codex, Cursor, etc.)
- **Curated Sources**: Skills from Anthropic, OpenAI, and community experts

## Skills Registry

### Skill Sources

Skills are curated from trusted GitHub repositories:
- **anthropics/skills** - Anthropic's official skill collection
- **openai/skills** - OpenAI's curated and system skills
- **mattpocock/skills** - Matt Pocock's development skills
- **MiniMax-AI/skills** - MiniMax's skill collection

### LLM-Based Classification

Skills are automatically classified into 10 domains:

| Domain | Description | Example Skills |
|--------|-------------|----------------|
| `planning-and-workflows` | Project planning, task management | `notion-spec-to-implementation` |
| `development` | Code development, SDKs, app building | `skill-creator`, `mcp-builder` |
| `design-and-media` | UI design, visual arts, creative work | `figma-*`, `canvas-design` |
| `documentation` | Document creation, formatting | `docx`, `pptx`, `pdf` |
| `data-and-research` | Data analysis, research | `jupyter-notebook` |
| `testing-and-debugging` | Testing, debugging, error handling | `playwright`, `sentry` |
| `deployment-and-infrastructure` | Cloud deployment, hosting | `netlify-deploy`, `render-deploy` |
| `integrations-and-tools` | API integrations, MCP servers | `mcp-builder`, `linear` |
| `collaboration-and-communication` | Team communication, meetings | `internal-comms`, `notion-meeting-intelligence` |
| `security-and-privacy` | Security reviews, threat modeling | `security-threat-model` |

Each skill has up to 3 tags describing specific capabilities.

### Registry Format

```toml
[skills.tdd]
name = "test-driven-development"
description = "Write tests before implementation code using TDD methodology"
source_url = "https://github.com/mattpocock/skills/tree/main/tdd"
stars = 1500
context_size = 2400
domain = "development"
last_updated = "2026-04-15"
tags = ["testing", "tdd", "best-practices"]
```

## Installation

### Homebrew (macOS)

```bash
brew tap ydeng11/rulesify
brew install rulesify
```

Or install directly:
```bash
brew install ydeng11/rulesify/rulesify
```

### One-liner (Linux/macOS)

```bash
curl -sSL https://github.com/ydeng11/rulesify/releases/latest/download/install.sh | bash
```

After installation, restart your shell or run:
```bash
export PATH="$HOME/.local/bin:$PATH"
```

### Manual Installation

1. Download the latest binary from [GitHub Releases](https://github.com/ydeng11/rulesify/releases)
2. Install to `~/.local/bin`:
   ```bash
   mkdir -p ~/.local/bin
   mv rulesify-<os>-<arch> ~/.local/bin/rulesify
   chmod +x ~/.local/bin/rulesify
   ```

## Quick Start

```bash
# List all available skills
rulesify skill list

# Filter by domain
rulesify skill list --domain development

# Search by tags
rulesify skill list --tags testing,debugging

# View skill details
rulesify skill show playwright

# Install a skill
rulesify skill add test-driven-development
```

## Command Reference

### Skills Commands

| Command | Description |
|---------|-------------|
| `rulesify init` | Interactive setup - select tools and skills |
| `rulesify skill list` | List installed skills |
| `rulesify skill add <skill-id>` | Download and install a skill (project level) |
| `rulesify skill add <skill-id> --global` | Install a skill globally |
| `rulesify skill remove <skill-id>` | Remove a skill (prompts for confirmation) |
| `rulesify skill update` | Update installed skills to latest versions |

### Global Options

- `--config <path>` - Use custom configuration file
- `--verbose` - Enable detailed output for debugging

## Usage Examples

### Discover Skills

```bash
# Browse all skills
rulesify skill list

# Find deployment skills
rulesify skill list --domain deployment-and-infrastructure

# Find debugging skills
rulesify skill list --tags debugging

# Find testing-related skills
rulesify skill list --tags testing
```

### Install Skills

```bash
# Initialize project with interactive selection
rulesify init

# Install a skill (downloads from GitHub, creates folders)
rulesify skill add test-driven-development

# Install globally (available in all projects)
rulesify skill add test-driven-development --global

# Remove a skill (prompts for confirmation)
rulesify skill remove test-driven-development

# Update installed skills
rulesify skill update
```

### Filter by Domain

```bash
# Development skills
rulesify skill list --domain development

# Design skills
rulesify skill list --domain design-and-media

# Security skills
rulesify skill list --domain security-and-privacy
```

## Update Registry

The registry updates weekly via GitHub Actions. To manually refresh:

```bash
# Requires GitHub and OpenRouter tokens
GITHUB_TOKEN=xxx OPENROUTER_API_KEY=xxx cargo run --bin update-registry

# Force re-classification
cargo run --bin update-registry -- --force

# Verbose mode
cargo run --bin update-registry -- --force -v

# Custom model (default: claude-3.5-haiku)
OPENROUTER_MODEL=openai/gpt-4o-mini cargo run --bin update-registry -- --force
```

## Development

### Project Structure

```
rulesify/
├── Cargo.toml
├── registry.toml           # Skills registry (LLM-classified)
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── bin/
│   │   └── update-registry.rs  # Registry automation
│   ├── cli/
│   │   └── skill.rs            # Skill commands
│   ├── models/
│   │   ├── domain.rs           # 10 domains enum
│   │   ├── skill.rs            # Skill model
│   │   └── registry.rs         # Registry model
│   ├── llm/
│   │   ├── client.rs           # OpenRouter client
│   │   ├── classifier.rs       # Batch classification
│   │   └── prompt.rs           # Prompts
│   ├── registry/
│   │   ├── source.rs           # GitHub sources
│   │   ├── github.rs           # GitHub API
│   │   ├── parser.rs           # SKILL.md parser
│   │   ├── scorer.rs           # Quality scoring
│   │   └── generator.rs        # TOML generator
│   └── tui/
│       └── skill_selector.rs   # Interactive selection
├── .github/
│   └── workflows/
│       └── update-registry.yml # Weekly updates
└── docs/
    └ plans/
```

### Testing

```bash
cargo test        # Run all 62 tests
cargo check       # Verify compilation
cargo clippy      # Lint
```

### Test Coverage

- **62 total tests** across all modules
- **7 executor tests** (URL parsing, install/uninstall)
- **10 domain tests** (enum parsing, validation)
- **24 registry tests** (skills catalog, GitHub API)
- **4 TUI tests** (interactive selection)

## License

[MIT License](./LICENSE)

Copyright (c) 2024 Rulesify Contributors