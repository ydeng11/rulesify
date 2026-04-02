# Integrations

## External APIs

**GitHub Releases API:**
- Purpose: Fetch latest release tag for installation script
- Endpoint: `https://api.github.com/repos/{owner}/{repo}/releases/latest`
- Used in: `install.sh` for version detection
- Auth: None (public API, subject to rate limits)
- Fallback: Uses redirect-based latest URL to avoid API limits

**GitHub Release Downloads:**
- Purpose: Binary distribution
- URL pattern: `https://github.com/{repo}/releases/download/{tag}/{asset}`
- Assets: `rulesify-{os}-{arch}.tar.gz` or `.zip` for Windows
- Authentication: None (public releases)

## Databases & Storage

**Local File System:**
- Primary storage: YAML files in `~/.rulesify/rules/`
- File format: `{rule-id}.urf.yaml`
- Implementation: `src/store/file_store.rs`
- Operations: load_rule, save_rule, list_rules, delete_rule

**In-Memory Store:**
- Purpose: Testing and temporary operations
- Implementation: `src/store/memory_store.rs`
- Uses HashMap for rule storage

**No External Database:**
- All data stored locally as YAML files
- No database connections or external data stores

## Authentication & Security

**No Authentication Required:**
- Local-only CLI tool
- No user accounts or sessions
- No API keys or tokens for core functionality

**GitHub Token (CI/CD Only):**
- Used in GitHub Actions workflows for releases
- Token: `secrets.GITHUB_TOKEN` (GitHub-provided)
- Scope: Creating releases, uploading assets

**File Permissions:**
- Binary installed to `~/.local/bin/` with execute permission
- Config directory: `~/.rulesify/` (user-owned)

## Third-Party Services

### AI Tool Integrations (Export Targets)

**Cursor IDE:**
- Output directory: `.cursor/rules/`
- File format: `.mdc` (Markdown with YAML frontmatter)
- Converter: `src/converters/cursor.rs`
- Features: YAML frontmatter with description, globs, alwaysApply
- Supports apply modes: always, intelligent, specific_files, manual

**Cline:**
- Output directory: `.clinerules/`
- File format: `.md` (plain Markdown)
- Converter: `src/converters/cline.rs`
- Features: Simple Markdown with headings

**Claude Code:**
- Output file: `CLAUDE.md` (single file in project root)
- File format: `.md` (Markdown)
- Converter: `src/converters/claude_code.rs`
- Features: Structured Markdown with sections

**Goose:**
- Output file: `.goosehints`
- File format: `.goosehints` (plain text with underlines)
- Converter: `src/converters/goose.rs`
- Features: Title underline format (= for main, - for sections)

### Rule Tracking System

**Embedded Rule IDs:**
- All deployed files contain hidden HTML comment: `<!-- rulesify-id: {id} -->`
- Purpose: Track rule origin for sync/import operations
- Implementation: `src/utils/rule_id.rs`

**Sync Feature:**
- Purpose: Reconcile deployed rules with URF source
- Status: Partially implemented (stub in `src/sync/synchronizer.rs`)
- Commands: `rulesify sync --dry-run`, `rulesify sync --rule`, `rulesify sync --tool`

## CI/CD & Deployment

**GitHub Actions Workflows:**
- `.github/workflows/tests.yml` - Test suite on push/PR
- `.github/workflows/release.yml` - Production releases on tags
- `.github/workflows/releases.yml` - Release management

**Build Targets:**
- `x86_64-unknown-linux-gnu` (Linux AMD64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS ARM/Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows AMD64)

**Release Process:**
- Triggered by version tags (v*)
- Test job runs first
- Creates GitHub release with changelog
- Builds cross-platform binaries
- Generates SHA256 checksums
- Uploads install.sh and CHANGELOG.md

**Installation Methods:**
```bash
curl -fsSL https://github.com/ydeng11/rulesify/releases/latest/download/install.sh | sh
```

## Webhooks & Callbacks

**Incoming:**
- None (local CLI tool)

**Outgoing:**
- None (no external notifications)

## Platform Dependencies

**System Requirements:**
- Operating System: Linux, macOS, Windows
- Architecture: x86_64 (AMD64) or ARM64
- Shell: bash/zsh/fish for installation script
- curl: Required for installation
- jq: Optional (fallback parsing available)

**Optional Dependencies:**
- `jq`: For JSON parsing in install script (fallback available)
- `$EDITOR`: For rule editing command

---

*Integration audit: 2026-04-02*