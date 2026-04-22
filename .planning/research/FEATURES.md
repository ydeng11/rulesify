# Feature Research

**Domain:** AI Agent/Skill Discovery and Registry Systems
**Researched:** 2026-04-02
**Confidence:** MEDIUM

## Feature Landscape

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| Search/Filter | Users need to find relevant skills among many options | MEDIUM | Keyword search minimum; faceted filtering ideal |
| Categories/Tags | Browsing by topic is standard discovery pattern | LOW | Standard taxonomy (language, framework, task type) |
| Descriptions | Users need to understand what a skill does before installing | LOW | Brief summary + longer README |
| Source Link | Users want to verify provenance and inspect code | LOW | GitHub repo URL required |
| Installation Instructions | Users need to know how to use the skill | MEDIUM | Skill-specific instructions for AI to follow |
| Compatibility Indicators | Users need to know if skill works with their tools | MEDIUM | Which AI tools supported (Cursor, Claude Code, Cline, Goose) |
| Name + ID | Unique identifier for each skill | LOW | Required for commands (add/remove/list) |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| Project Scanning | Auto-suggest relevant skills based on project context | HIGH | Detects language, framework, existing AI configs |
| Curated Registry | Trust signal; avoids typosquatting, abandoned packages | LOW | Manual curation vs open submission (out of scope per PROJECT.md) |
| Popularity Metrics | Social proof helps users choose between similar options | MEDIUM | Download count, GitHub stars, usage frequency |
| Tool Compatibility Matching | Filter skills to only show compatible ones | MEDIUM | User selects tools they use, registry filters accordingly |
| Interactive Terminal UI | Better UX for multi-select than numbered lists | MEDIUM | ratatui or similar TUI library |
| Usage Examples | Shows how to use skill in practice | LOW | Code snippets, common patterns |
| Skill Dependencies | Some skills require other skills | MEDIUM | Explicit dependency declaration |
| Last Updated Date | Helps assess freshness and maintenance | LOW | GitHub commit timestamp |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| User-Contributed Registry | Community growth, more skills | Typosquatting, abandoned skills, security risks, curation burden | Curated registry with contribution PR process |
| Version Locking | Reproducibility, stability | Adds complexity, skills should be backward compatible | Fetch latest; skill authors maintain compatibility |
| Direct Skill Execution | One-click install, simpler UX | Skill-specific install processes vary widely | Generate instructions for AI to execute |
| Security Scanning | Malware detection, trust | Complex, requires security expertise, false positives | Trust curated sources, rely on GitHub reputation |
| Ratings/Reviews | Social proof, quality signals | Review bombing, gaming, requires moderation | Popularity metrics (downloads, stars) instead |
| Skill Forking/Modification | Customize skills | Fragmentation, no update path | Fork repo, add to separate registry entry |
| Complex Dependency Resolution | npm-like dependency tree | Over-engineering for current scale | Simple requires field, manual resolution |

## Feature Dependencies

```
[Project Scanning]
    └──requires──> [Tool Compatibility Detection]
                       └──requires──> [AI Tool Config Parsers]

[Interactive TUI]
    └──requires──> [Search/Filter]
    └──requires──> [Categories/Tags]

[Skill Installation]
    └──requires──> [Installation Instructions]
    └──requires──> [Source Link]

[Tool Compatibility Matching]
    └──requires──> [Compatibility Indicators in Registry]
    └──enhances──> [Project Scanning]

[Curated Registry]
    └──conflicts──> [User-Contributed Registry] (mutually exclusive models)
```

### Dependency Notes

- **Project Scanning requires Tool Compatibility Detection:** Must know which AI tools are configured to suggest relevant skills
- **Interactive TUI requires Search/Filter and Categories:** TUI without search/filter would be frustrating
- **Tool Compatibility Matching enhances Project Scanning:** Filter suggestions to only compatible skills
- **Curated Registry conflicts with User-Contributed Registry:** Different trust models, governance approaches

## MVP Definition

### Launch With (v1)

Minimum viable product - what's needed to validate the concept.

- [x] **Registry Data Structure** - JSON/TOML with skill metadata (name, description, source URL, tags, compatibility) - Essential foundation
- [x] **Search/Filter** - Keyword search and tag-based filtering - Users cannot discover skills without this
- [x] **Categories/Tags** - Language, framework, task type taxonomy - Standard discovery pattern
- [x] **Source Link** - GitHub repo URL for each skill - Trust and verification
- [x] **Installation Instructions** - Skill-specific instructions for AI - Core value proposition
- [x] **Compatibility Indicators** - Which AI tools each skill supports - Users need to know if skill works for them
- [x] **Basic Commands** - init, add, remove, list skills - Core CLI functionality

### Add After Validation (v1.x)

Features to add once core is working.

- [ ] **Project Scanning** - Detect project type, suggest relevant skills - Major UX improvement
- [ ] **Interactive TUI** - Better multi-select experience - Validated need for better UX
- [ ] **Popularity Metrics** - GitHub stars, download counts - Social proof for choosing skills
- [ ] **Tool Compatibility Detection** - Auto-detect which AI tools user has - Reduces friction in setup
- [ ] **Last Updated Date** - Freshness indicator - Helps assess maintenance status

### Future Consideration (v2+)

Features to defer until product-market fit is established.

- [ ] **Skill Dependencies** - Some skills require others - Adds complexity
- [ ] **Usage Examples** - Code snippets, patterns - Nice to have but not essential
- [ ] **Registry API** - Programmatic access - Unknown if needed
- [ ] **Offline Cache** - Work without network - Edge case for MVP

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Registry Data Structure | HIGH | LOW | P1 |
| Search/Filter | HIGH | MEDIUM | P1 |
| Categories/Tags | HIGH | LOW | P1 |
| Source Link | HIGH | LOW | P1 |
| Installation Instructions | HIGH | MEDIUM | P1 |
| Compatibility Indicators | HIGH | MEDIUM | P1 |
| Basic Commands | HIGH | MEDIUM | P1 |
| Project Scanning | HIGH | HIGH | P2 |
| Interactive TUI | MEDIUM | MEDIUM | P2 |
| Popularity Metrics | MEDIUM | MEDIUM | P2 |
| Tool Compatibility Detection | MEDIUM | MEDIUM | P2 |
| Last Updated Date | LOW | LOW | P2 |
| Skill Dependencies | MEDIUM | HIGH | P3 |
| Usage Examples | MEDIUM | LOW | P3 |
| Registry API | LOW | MEDIUM | P3 |
| Offline Cache | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for launch
- P2: Should have, add when possible
- P3: Nice to have, future consideration

## Competitor Feature Analysis

| Feature | cursor.directory | MCP Servers | npm Registry | Our Approach |
|---------|------------------|-------------|--------------|--------------|
| Search | Basic keyword | GitHub search only | Advanced with filters | Keyword + tag filter |
| Categories | Manual categories | Reference vs Third-party | Keywords, maintainers | Curated taxonomy |
| Trust Signals | None (community) | Official badge | Download counts, badges | Curated sources only |
| Compatibility | Cursor only | Protocol version | Node version, engines | AI tool compatibility |
| Installation | Copy-paste | Manual config | npm install | AI-executed instructions |
| Popularity | Stars only | Stars | Downloads, stars, quality score | GitHub stars |
| Version | None (latest) | None (latest) | Full semver | Latest only |
| Governance | Open PR | Open PR + Official section | Open publish | Curated PR process |

## Discovery Patterns Observed

### How Users Currently Discover Skills/Agents

1. **Word of Mouth / Social Media** - Twitter/X, Discord communities share rules
2. **Awesome Lists** - GitHub awesome-cursorrules style curated lists
3. **Directory Sites** - cursor.directory, smithery.ai for MCP servers
4. **GitHub Search** - Searching for `.cursorrules` or `agents.md` files
5. **Documentation** - Tool docs link to recommended skills/extensions

### Common Pain Points

1. **No Central Registry** - Skills scattered across repos, gists, tweets
2. **Trust Unknown** - Is this skill safe? Well-maintained? Compatible?
3. **Installation Friction** - Manual copy-paste, config editing
4. **Compatibility Uncertainty** - Does this work with my AI tool?
5. **Quality Unknown** - Is this skill any good? Maintained?
6. **Search Difficulty** - Hard to find relevant skills for specific needs

## Sources

- [cursor.directory](https://cursor.directory/) - Cursor rules directory
- [MCP Servers Registry](https://github.com/modelcontextprotocol/servers) - MCP server reference implementations
- [agents.md](https://agents.md) - Agent file specification
- npm package registry discovery patterns (training knowledge)
- VS Code extension marketplace patterns (training knowledge)
- Homebrew formulae discovery patterns (training knowledge)

---
*Feature research for: AI Agent/Skill Discovery and Registry Systems*
*Researched: 2026-04-02*