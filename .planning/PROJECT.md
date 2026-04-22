# Agents & Skills Auto-Loading Feature

## What This Is

A feature for rulesify CLI that helps AI agents self-configure with the right tools by discovering and installing agents files and skills from curated GitHub repositories. Users run `rulesify init`, the tool scans their project, suggests relevant agents/skills, and generates instructions for the AI to follow.

## Core Value

**Reduce friction for AI agent setup.** Users shouldn't have to manually search for, discover, and configure agent instructions and skills. Rulesify acts as a trusted curator and discovery mechanism.

## Requirements

### Validated

(None yet — shipping to validate)

### Active

- [ ] `rulesify init` command that scans project and suggests relevant agents/skills
- [ ] Project scanning detects language, framework, existing AI tool configurations
- [ ] Ask user which AI tools they use (Cursor, Claude Code, Cline, Goose, etc.)
- [ ] Interactive terminal UI for selecting agents/skills from suggestions
- [ ] Registry of curated agents/skills with metadata (tags, popularity, descriptions, tool compatibility)
- [ ] Generate tailored installation instructions for AI to execute
- [ ] Skills fetched from GitHub repos (latest, no version locking)
- [ ] Commands to add/remove/list skills after initial setup
- [ ] Registry stored in rulesify repo, fetched at runtime

### Out of Scope

- Executing installation directly — AI agent handles the actual install following skill-specific instructions
- Version locking or release-based skill distribution
- User-contributed/external registries beyond the curated list
- Skill validation or security scanning (trust the curated sources)

## Context

**Existing codebase:** Rulesify is a Rust CLI tool for managing Universal Rule Files (URF). It already has:
- Rule management (create, deploy, import, validate, sync)
- Support for 4 AI tools: Cursor, Cline, Claude Code, Goose
- Converter pattern for bidirectional format translation
- GitHub Actions for CI/CD

**Agent file definition:** Per https://agents.md — standardized agent definition files

**Skill definition:** Folders containing instructions, scripts, and resources that AI tools load dynamically for specialized tasks. Each skill has its own installation instructions that the AI follows.

**Design decision:** Rulesify does NOT install skills directly. Instead, it:
1. Curates a registry of trusted skill sources
2. Helps users discover relevant skills
3. Generates instructions telling the AI where to find and how to install each skill

This keeps rulesify simple while letting skill authors control their own installation process.

## Constraints

- **Tech stack:** Rust (existing codebase), must integrate with current clap-based CLI
- **Runtime:** No external dependencies beyond what's already in Cargo.toml
- **GitHub access:** Must work with public GitHub repos (raw file URLs, repo browsing)
- **Rate limits:** GitHub API has 60 requests/hr unauthenticated; design should minimize API calls

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Registry in rulesify repo | Single source of truth, easy to update, curated quality | — Pending |
| Fetch latest only | Simplicity over version control; skills should be backward compatible | — Pending |
| AI executes installation | Skill authors control their install process; rulesify stays simple | — Pending |
| Interactive terminal UI | Better UX for multi-select than numbered lists | — Pending |

---

*Last updated: 2026-04-02 after initialization*

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state