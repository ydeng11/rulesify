# Tech Stack

## Languages & Runtime

**Primary Language:**
- Rust (Edition 2021) - Main implementation language
- Version managed via Cargo.lock, currently v0.3.1

**Runtime Environment:**
- Native binary compilation (no runtime required)
- Build tool: Cargo (Rust's official build system and package manager)
- Lockfile: `Cargo.lock` present for dependency pinning

## Frameworks & Libraries

**Core Frameworks:**
- clap v4.5.1 - CLI argument parsing with derive macros
- clap_complete v4.5.1 - Shell completion generation
- tokio v1.37 - Async runtime (rt-multi-thread, macros features)

**Serialization/Data:**
- serde v1.0 - Serialization framework (derive feature)
- serde_yaml v0.9 - YAML serialization for rule files (.urf.yaml)
- serde_json v1.0 - JSON serialization for metadata/tool_overrides
- chrono v0.4 - Date/time handling with serde support

**Error Handling:**
- anyhow v1.0 - Application error handling with context
- thiserror v1.0 - Custom error type derivation

**File System & Utilities:**
- glob v0.3 - Pattern matching for file discovery
- walkdir v2.4 - Recursive directory traversal
- dirs v5.0 - System directory paths (home directory detection)
- regex v1.10 - Regular expression matching

**Logging:**
- log v0.4 - Logging facade
- env_logger v0.10 - Log implementation with env filtering

**Development Dependencies:**
- tempfile v3.8 - Temporary file/directory creation for tests

## Build & Configuration

**Build Commands:**
```bash
cargo build              # Development build
cargo build --release    # Optimized production build
cargo test               # Run test suite
```

**Configuration Files:**
- `Cargo.toml` - Project manifest, dependencies, metadata
- `Cargo.lock` - Dependency lockfile (committed)
- `.gitignore` - Excludes target/, IDE files

**Application Configuration:**
- Global config: `~/.rulesify/config.yaml`
- Rules directory: `~/.rulesify/rules/` (default)
- Config structure defined in `src/models/config.rs`
- Editor: Uses `$EDITOR` environment variable when set

**Environment Variables:**
- `EDITOR` - Preferred text editor for rule editing
- `RULESIFY_REPO` - Override GitHub repo for install script

## Key Technical Decisions

**CLI Architecture:**
- Uses clap derive macros for command structure
- Subcommand pattern: rule, deploy, import, validate, sync, config, completion
- Global flags: `--config` for custom config path, `--verbose` for debug output
- Entry point: `src/main.rs` with `env_logger::init()` for logging

**Rule Storage Format:**
- Universal Rule File (URF) format: `.urf.yaml` files
- YAML-based with structured metadata and content sections
- Supports tool-specific overrides via `tool_overrides` HashMap

**Converter Pattern:**
- Trait-based design: `RuleConverter` trait in `src/converters/mod.rs`
- Each AI tool has dedicated converter module
- Bidirectional conversion: to/from tool-specific formats

**Async Runtime:**
- Tokio multi-threaded runtime for potential async operations
- Currently used for macros feature; most operations are synchronous

**Error Handling Strategy:**
- anyhow for application-level error propagation
- Context chains for debugging: `.with_context(|| format!(...))`
- Error source chain printed in CLI on failure

**Binary Distribution:**
- Cross-platform builds via GitHub Actions
- Targets: linux-amd64, darwin-amd64, darwin-arm64, windows-amd64
- Installation via curl + shell script from GitHub releases

---

*Stack analysis: 2026-04-02*