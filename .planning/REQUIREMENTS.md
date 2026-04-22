# Requirements: Agents & Skills Auto-Loading

**Defined:** 2026-04-03
**Core Value:** Reduce friction for AI agent setup through discovery and installation of community-validated agents/skills

## v1 Requirements

### Registry

- [ ] **REG-01**: Built-in registry file with curated list of agents/skills from GitHub
- [ ] **REG-02**: Registry entries include: id, name, description, source (GitHub URL), tags, tool compatibility, popularity metrics
- [ ] **REG-03**: Registry stored in rulesify repo, fetched at runtime (always latest, no version locking)
- [ ] **REG-04**: Support for both agent files (agents.md format) and skill folders

### Project Scanning

- [ ] **SCAN-01**: Detect project language(s) from file extensions and config files
- [ ] **SCAN-02**: Detect framework(s) from package files (package.json, Cargo.toml, etc.)
- [ ] **SCAN-03**: Detect existing AI tool configurations (.cursor/rules/, CLAUDE.md, .clinerules, .goosehints)
- [ ] **SCAN-04**: Identify project type tags for skill matching

### Interactive Setup

- [ ] **SETUP-01**: `rulesify init` command triggers setup flow
- [ ] **SETUP-02**: Ask user which AI tools they use (Cursor, Claude Code, Cline, Goose)
- [ ] **SETUP-03**: Scan project context automatically
- [ ] **SETUP-04**: Match project context against registry tags
- [ ] **SETUP-05**: Interactive terminal UI (arrow keys, space to select, enter to confirm)
- [ ] **SETUP-06**: Show skill name, description, compatibility, popularity
- [ ] **SETUP-07**: Generate installation instructions for AI to execute

### Installation Output

- [ ] **INST-01**: Output tailored instructions based on selected AI tools
- [ ] **INST-02**: Include GitHub source URL for each selected skill
- [ ] **INST-03**: Provide skill-specific installation steps from skill's own instructions
- [ ] **INST-04**: Instructions tell AI where to create agent.md or skill files

### Lifecycle Management

- [ ] **LIFE-01**: `rulesify skill list` shows installed skills
- [ ] **LIFE-02**: `rulesify skill add <name>` adds new skill from registry
- [ ] **LIFE-03**: `rulesify skill remove <name>` removes skill and provides cleanup instructions
- [ ] **LIFE-04**: Track installed skills in project config or manifest file

### User Experience

- [ ] **UX-01**: Clear error messages when registry fetch fails
- [ ] **UX-02**: Offline fallback to cached/built-in registry
- [ ] **UX-03**: Verbose mode shows detailed matching logic
- [ ] **UX-04**: Help text explains agents vs skills distinction

## v2 Requirements

Deferred to future release.

### Enhanced Discovery

- **DISC-01**: Search/filter registry by keyword
- **DISC-02**: Browse registry by category
- **DISC-03**: Show trending/popular skills

### Extended Features

- **EXT-01**: User-configured external registry sources
- **EXT-02**: Custom skill creation from templates
- **EXT-03**: Skill update notifications
- **EXT-04**: Integration with existing rulesify rule system

## Out of Scope

| Feature | Reason |
|---------|--------|
| Direct skill execution | Rulesify generates instructions, AI executes |
| Version locking | Always fetch latest for simplicity |
| User-contributed registry | Curated list only for trust |
| Skill validation/security scanning | Trust curated sources |
| Rating/review system | Not needed for curated list |
| Private/internal registries | Focus on public GitHub sources first |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| REG-01 | Phase 1 | Pending |
| REG-02 | Phase 1 | Pending |
| REG-03 | Phase 1 | Pending |
| REG-04 | Phase 1 | Pending |
| SCAN-01 | Phase 1 | Pending |
| SCAN-02 | Phase 1 | Pending |
| SCAN-03 | Phase 1 | Pending |
| SCAN-04 | Phase 1 | Pending |
| SETUP-01 | Phase 2 | Pending |
| SETUP-02 | Phase 2 | Pending |
| SETUP-03 | Phase 2 | Pending |
| SETUP-04 | Phase 2 | Pending |
| SETUP-05 | Phase 2 | Pending |
| SETUP-06 | Phase 2 | Pending |
| SETUP-07 | Phase 2 | Pending |
| INST-01 | Phase 2 | Pending |
| INST-02 | Phase 2 | Pending |
| INST-03 | Phase 2 | Pending |
| INST-04 | Phase 2 | Pending |
| LIFE-01 | Phase 3 | Pending |
| LIFE-02 | Phase 3 | Pending |
| LIFE-03 | Phase 3 | Pending |
| LIFE-04 | Phase 3 | Pending |
| UX-01 | Phase 3 | Pending |
| UX-02 | Phase 3 | Pending |
| UX-03 | Phase 3 | Pending |
| UX-04 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 26 total
- Mapped to phases: 26
- Unmapped: 0 ✓

---
*Requirements defined: 2026-04-03*
*Last updated: 2026-04-03 after initialization*