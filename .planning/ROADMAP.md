# Roadmap: Agents & Skills Auto-Loading

**Created:** 2026-04-02
**Milestone:** Agents & Skills Auto-Loading
**Granularity:** coarse

## Core Value

Reduce friction for AI agent setup through discovery and installation of community-validated agents/skills.

## Phases

- [x] **Phase 1: Registry & Project Scanning** - Foundation layer: built-in registry data model and project context detection
- [x] **Phase 2: Interactive Setup Flow** - Main user journey: `rulesify init` command with interactive skill selection
- [x] **Phase 3: Lifecycle & UX Polish** - Ongoing management: skill commands and user experience refinements

---

## Phase Details

### Phase 1: Registry & Project Scanning
**Goal**: Users can access a curated registry of agents/skills and rulesify can detect project context automatically
**Depends on**: Nothing (first phase)
**Requirements**: REG-01, REG-02, REG-03, REG-04, SCAN-01, SCAN-02, SCAN-03, SCAN-04
**Success Criteria** (what must be TRUE):
  1. User can fetch and view the curated registry of agents/skills from the rulesify repo
  2. Registry entries display id, name, description, source URL, tags, tool compatibility, and popularity metrics
  3. User sees clear error message when registry fetch fails (with offline fallback available)
  4. Running project scan detects programming language(s) from file extensions and config files
  5. Running project scan identifies frameworks from package files (package.json, Cargo.toml, etc.)
  6. Running project scan detects existing AI tool configurations (.cursor/rules/, CLAUDE.md, .clinerules, .goosehints)
**Plans**: TBD

### Phase 2: Interactive Setup Flow
**Goal**: Users can run `rulesify init` to get personalized skill recommendations based on their project
**Depends on**: Phase 1
**Requirements**: SETUP-01, SETUP-02, SETUP-03, SETUP-04, SETUP-05, SETUP-06, SETUP-07, INST-01, INST-02, INST-03, INST-04
**Success Criteria** (what must be TRUE):
  1. User runs `rulesify init` and is prompted to select which AI tools they use
  2. User sees their project context automatically detected (languages, frameworks, existing AI configs)
  3. User can browse and select skills using arrow keys and space bar in an interactive terminal UI
  4. User sees skill name, description, tool compatibility, and popularity for each option
  5. User receives tailored installation instructions with GitHub URLs and skill-specific steps
  6. Instructions clearly state where to create agent.md or skill files for each selected tool
**Plans**: TBD
**UI hint**: yes

### Phase 3: Lifecycle & UX Polish
**Goal**: Users can manage installed skills after initial setup with clear feedback and error handling
**Depends on**: Phase 2
**Requirements**: LIFE-01, LIFE-02, LIFE-03, LIFE-04, UX-01, UX-02, UX-03, UX-04
**Success Criteria** (what must be TRUE):
  1. User can run `rulesify skill list` to see all installed skills
  2. User can run `rulesify skill add <name>` to install a new skill from the registry
  3. User can run `rulesify skill remove <name>` to uninstall a skill with cleanup instructions
  4. User sees clear error messages when registry fetch fails with offline fallback
  5. User can run `--verbose` to see detailed matching logic for skill suggestions
  6. User can access help text explaining the distinction between agents and skills
**Plans**: TBD
**UI hint**: yes

---

## Progress

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Registry & Project Scanning | 1/1 | Complete | 2026-04-09 |
| 2. Interactive Setup Flow | 1/1 | Complete | 2026-04-09 |
| 3. Lifecycle & UX Polish | 1/1 | Complete | 2026-04-09 |

---

## Coverage

| Requirement | Phase | Status |
|-------------|-------|--------|
| REG-01 | Phase 1 | Delivered |
| REG-02 | Phase 1 | Delivered |
| REG-03 | Phase 1 | Delivered |
| REG-04 | Phase 1 | Delivered |
| SCAN-01 | Phase 1 | Delivered |
| SCAN-02 | Phase 1 | Delivered |
| SCAN-03 | Phase 1 | Delivered |
| SCAN-04 | Phase 1 | Delivered |
| SETUP-01 | Phase 2 | Delivered |
| SETUP-02 | Phase 2 | Delivered |
| SETUP-03 | Phase 2 | Delivered |
| SETUP-04 | Phase 2 | Delivered |
| SETUP-05 | Phase 2 | Delivered |
| SETUP-06 | Phase 2 | Delivered |
| SETUP-07 | Phase 2 | Delivered |
| INST-01 | Phase 2 | Delivered |
| INST-02 | Phase 2 | Delivered |
| INST-03 | Phase 2 | Delivered |
| INST-04 | Phase 2 | Delivered |
| LIFE-01 | Phase 3 | Delivered |
| LIFE-02 | Phase 3 | Delivered |
| LIFE-03 | Phase 3 | Delivered |
| LIFE-04 | Phase 3 | Delivered |
| UX-01 | Phase 3 | Delivered |
| UX-02 | Phase 3 | Delivered |
| UX-03 | Phase 3 | Delivered |
| UX-04 | Phase 3 | Delivered |

**Total:** 26/26 requirements delivered (100%)

---

*Roadmap created: 2026-04-02*