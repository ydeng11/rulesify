# AGENTS.md - Rulesify Project Guide

> Quick reference for AI agents working on this Rust CLI project.

---

## Tech Stack

### Core
- **Rust** 1.94+ (Edition 2021)
- **Cargo** - Build system and package manager

### Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| `clap` | 4.5 | CLI argument parsing (derive macros) |
| `tokio` | 1.37 | Async runtime (rt-multi-thread, macros) |
| `reqwest` | 0.12 | HTTP client for registry fetch (json feature) |
| `serde` | 1.0 | Serialization (derive) |
| `toml` | 0.8 | TOML parsing for config and registry |
| `ratatui` | 0.26 | Terminal UI framework |
| `crossterm` | 0.27 | Cross-platform terminal control |
| `chrono` | 0.4 | Date/time with serde support |
| `anyhow` | 1.0 | Application error handling |
| `thiserror` | 1.0 | Custom error types |
| `dirs` | 5.0 | System directory paths |
| `walkdir` | 2.4 | Recursive directory traversal |
| `env_logger` | 0.10 | Logging implementation |
| `log` | 0.4 | Logging facade |

### Dev Dependencies
- `tempfile` 3.8 - Temporary files for tests

---

## Project Structure

```
rulesify/
├── Cargo.toml              # Dependencies and metadata
├── registry.toml           # Built-in skill registry (compiled into binary)
├── src/
│   ├── main.rs             # Entry point (tokio runtime, CLI dispatch)
│   ├── lib.rs              # Library exports
│   ├── cli/
│   │   ├── mod.rs          # CLI structure (clap Parser/Subcommand)
│   │   ├── init.rs         # `rulesify init` interactive setup
│   │   └── skill.rs        # `rulesify skill list/add/remove/update`
│   ├── models/
│   │   ├── mod.rs          # Model exports
│   │   ├── skill.rs        # Skill struct (name, description, tags, tools)
│   │   ├── registry.rs     # Registry struct (HashMap of skills)
│   │   ├── context.rs      # ProjectContext (languages, frameworks, tools)
│   │   ├── config.rs       # ProjectConfig (installed skills tracking)
│   │   └── *_tests.rs      # Unit tests
│   ├── registry/
│   │   ├── mod.rs          # Registry module exports
│   │   ├── data.rs         # Built-in registry loader (include_str!)
│   │   ├── fetch.rs        # GitHub registry fetcher (async)
│   │   ├── cache.rs        # Local cache management
│   │   └ *_tests.rs        # Unit tests
│   ├── scanner/
│   │   ├── mod.rs          # Scanner orchestration
│   │   ├── language.rs     # Language detection (file extensions + configs)
│   │   ├── framework.rs    # Framework detection (package files)
│   │   ├── tool_config.rs  # AI tool config detection (.cursor, CLAUDE.md)
│   │   └ *_tests.rs        # Unit tests
│   ├── tui/
│   │   ├── mod.rs          # TUI module exports
│   │   ├── tool_picker.rs  # Interactive tool selection (ratatui)
│   │   └ skill_selector.rs # Interactive skill selection (ratatui)
│   ├── installer/
│   │   ├── mod.rs          # Installer module exports
│   │   └ instructions.rs   # Installation instruction generator
│   └ utils/
│       ├── mod.rs          # Utils exports
│       └ error.rs          # Error types (RulesifyError enum)
├── .planning/              # GSD project management files
│   ├── STATE.md            # Current project state
│   ├── ROADMAP.md          # Phase breakdown
│   ├── PROJECT.md          # Project description
│   └ REQUIREMENTS.md       # Requirements list
│   └ codebase/             # Codebase documentation
│   └ research/             # Research artifacts
├── docs/
│   └ plans/                # Execution plans and summaries
└── target/                 # Build output (release/debug)
```

---

## Common Commands

### Build & Run

```bash
# Check compilation (fast)
cargo check

# Build debug version
cargo build

# Build optimized release version
cargo build --release

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run -- init

# Run release binary
./target/release/rulesify --help
./target/release/rulesify init
./target/release/rulesify skill list
```

### Development

```bash
# Format code
cargo fmt

# Lint with clippy
cargo clippy

# Generate documentation
cargo doc --open

# Watch for changes (requires cargo-watch)
cargo watch -x check
```

### CLI Commands

```bash
# Interactive setup
rulesify init

# List installed skills
rulesify skill list

# Add skill from registry
rulesify skill add test-driven-development

# Remove installed skill
rulesify skill remove test-driven-development

# Update registry cache
rulesify skill update

# Verbose mode
rulesify -v skill list
```

---

## Standards

### Code Style

1. **No comments** unless explicitly requested
   - Code should be self-documenting through clear naming
   - Complex logic may have brief inline comments

2. **Formatting**
   - Use `cargo fmt` before committing
   - Default Rust style: 4-space indentation, max 100 chars
   - Opening braces on same line

3. **Naming Conventions**
   - Functions/variables: `snake_case`
   - Types/structs/enums: `PascalCase`
   - Constants: `SCREAMING_SNAKE_CASE`
   - Modules: `snake_case` (directory names match)

### Error Handling

1. **Use `anyhow::Result<T>`** for all fallible operations
2. **Use `.context()`** to add descriptive error messages:
   ```rust
   fs::read_to_string(path)
       .with_context(|| format!("Failed to read config: {}", path))?
   ```
3. **Use `thiserror`** for custom error types (see `src/utils/error.rs`)
4. **Early returns** with `?` operator

### Async Pattern

1. **Tokio runtime** is available but most operations are synchronous
2. **Use `async fn`** for:
   - Network operations (registry fetch)
   - Operations that may benefit from async in future
3. **Mark main with `#[tokio::main]`**

### Testing

1. **Unit tests** in same module with `_tests.rs` suffix:
   ```
   src/models/skill.rs
   src/models/skill_tests.rs
   ```
2. **Test imports** use `crate::` not `super::`:
   ```rust
   use crate::models::Skill;  // Correct
   use super::Skill;          // Wrong (fails for test modules)
   ```
3. **Use `tempfile`** for tests needing file operations
4. **Run tests** before claiming work complete

### Module Organization

1. **Each module has `mod.rs`** exposing public types:
   ```rust
   pub mod skill;
   pub use skill::Skill;
   ```
2. **Private implementations** in separate files
3. **Tests** conditionally compiled with `#[cfg(test)]`

### Logging

1. **Use `log` crate** with macros: `debug!`, `info!`, `error!`
2. **Initialize in main**: `env_logger::init()`
3. **Enable with env var**: `RUST_LOG=debug cargo run`

### Git Commits

1. **Atomic commits** - each task commits independently
2. **Conventional commit format**:
   ```
   feat: add feature
   fix: fix bug
   test: add tests
   refactor: code cleanup
   docs: documentation
   chore: tooling/config
   ```
3. **Stage files individually** - never `git add .`
4. **Run tests** before committing

### TOML Data Files

1. **Registry** (`registry.toml`):
   - Compiled into binary with `include_str!`
   - Fallback when network unavailable
   
2. **Config** (`~/.rulesify/config.yaml`):
   - User settings (future)
   
3. **Project config** (`./.rulesify.toml`):
   - Installed skills tracking
   - Selected AI tools

---

## Important Files

| File | Purpose | Edit Frequency |
|------|---------|----------------|
| `Cargo.toml` | Dependencies | Low (add new deps) |
| `registry.toml` | Skill registry | Medium (add skills) |
| `src/cli/mod.rs` | CLI structure | Low (new commands) |
| `src/models/*.rs` | Data structures | Medium |
| `src/utils/error.rs` | Error types | Low |
| `.planning/STATE.md` | Project state | High (update per task) |

---

## Before Making Changes

1. **Run `cargo check`** to ensure current state compiles
2. **Understand module dependencies** - check `mod.rs` exports
3. **Check similar code** for patterns to follow
4. **Run tests** after changes: `cargo test`
5. **Format code**: `cargo fmt`
6. **Update STATE.md** if completing a phase/plan

---

## Key Architectural Decisions

1. **Layered modules** - cli → models → utils (no circular deps)
2. **Built-in fallback** - registry: builtin → fetch → cache
3. **Separation of concerns**:
   - Scanner: detects context (languages, frameworks, tools)
   - Registry: provides skill metadata
   - TUI: interactive selection
   - Installer: generates instructions (AI executes them)
4. **No auto-installation** - rulesify generates instructions, AI agent executes
5. **Terminal UI** - ratatui for interactive selection (arrow keys, space, enter)

---

## Verification Before Completion

Always run these before claiming work is done:

```bash
cargo check    # Must pass
cargo test     # Must pass
cargo fmt      # Format code
cargo clippy   # Optional but recommended
```

---

*Last updated: 2026-04-09*