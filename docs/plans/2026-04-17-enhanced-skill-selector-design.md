# Design: Enhanced Skill Selector with Filtering & Sorting

## Overview

Replace the current `SkillSelector` TUI with a richer interface that:
- Shows all skills by default (no auto-filtering)
- Provides domain tabs for quick filtering
- Offers tag multi-select via popup
- Supports sorting by stars/score/name

## TUI Layout

```
┌─ Select Skills ────────────────────────────────────────────────────┐
│ [All] [Development] [Design] [Documentation] [Security] [Deploy]   │  <- Domain tabs
│────────────────────────────────────────────────────────────────────│
│ Sort: Stars ↓ | Tags: figma, deployment                            │  <- Status bar
│────────────────────────────────────────────────────────────────────│
│ > [x] figma-use - Figma JS execution (★16,872)                     │
│   [ ] figma-implement-design - Implement from Figma (★16,872)     │
│   [ ] cli-creator - Build composable CLI (★16,872)                 │
│   [ ] render-deploy - Deploy to Render (★16,872)                   │
│   ...                                                               │
│────────────────────────────────────────────────────────────────────│
│ ↑↓ Nav | Space Select | ←→ Domain | s Sort | t Tags | Enter Done │  <- Help bar
└────────────────────────────────────────────────────────────────────┘
```

**Sections:**
1. Domain tabs (horizontal row at top)
2. Status bar (current sort + active tags)
3. Skill list (scrollable, multi-select)
4. Help bar (key bindings)

## Domain Tabs

- Keys: `←` / `→` to navigate between tabs
- Display: Show domain names extracted from registry (unique values)
- "All" tab: Shows all skills (default view)
- Selection: Single-select (only one domain active at a time)
- Effect: Filters skill list to show only skills matching selected domain

**Domain list from registry:**
- All
- deployment-and-infrastructure
- development
- design-and-media
- documentation
- security-and-privacy
- testing-and-debugging
- collaboration-and-communication
- data-and-research
- planning-and-workflows

## Tag Filtering

- Key: `t` opens tag selection popup
- Popup UI:
  ```
  ┌─ Select Tags ────────────────────────────┐
  │ [x] figma                                │
  │ [x] deployment                           │
  │ [ ] api-integration                      │
  │ [ ] cli-tools                            │
  │ [ ] documentation                        │
  │ ...                                      │
  │──────────────────────────────────────────│
  │ ↑↓ Nav | Space Toggle | Enter Apply     │
  └──────────────────────────────────────────┘
  ```
- Logic: AND filtering - skill must have ALL selected tags
- Tags source: Extracted from registry (unique values, sorted by frequency)
- Apply: On `Enter`, popup closes and list updates

## Sorting

- Key: `s` cycles through modes:
  1. Stars ↓ (default)
  2. Stars ↑
  3. Score ↓
  4. Score ↑
  5. Name A-Z
  6. Name Z-A
- Display: Status bar shows current sort mode
- Effect: Immediately re-orders the skill list

## Key Bindings

| Key | Action |
|-----|--------|
| `↑/↓` | Navigate skill list |
| `Space` | Toggle skill selection |
| `←/→` | Navigate domain tabs |
| `s` | Cycle sort mode |
| `t` | Open tag filter popup |
| `Enter` | Confirm selection / Apply tag filter |
| `Esc` | Cancel (exit) |

## Architecture Changes

### New Components

1. `SkillSelectorState` - State machine tracking:
   - Current domain tab index
   - Selected tags (HashSet)
   - Current sort mode
   - Skill list (filtered & sorted)
   - Selected skill indices
   - Current skill index

2. `DomainTabs` widget - Horizontal tab selector

3. `TagPopup` - Modal for multi-selecting tags

4. `StatusBar` - Shows sort mode + active tags

5. `HelpBar` - Key binding hints

### State Flow

```
Registry → extract domains → extract tags → initial state
         ↓
User interacts (domain nav, sort cycle, tag popup)
         ↓
Update filters → apply filter + sort → update skill list
         ↓
Render updated UI
         ↓
User selects skills → Enter → return selected
```

### Filter Logic

```rust
fn apply_filters(&mut self) {
    let filtered: Vec<_> = self.all_skills.iter()
        .filter(|(_, skill)| {
            // Domain filter (if not "All")
            if self.domain_index > 0 {
                let domain = self.domains[self.domain_index];
                skill.domain == domain
            } else {
                true
            }
        })
        .filter(|(_, skill)| {
            // Tag filter (AND logic)
            self.selected_tags.iter().all(|tag| skill.tags.contains(tag))
        })
        .collect();

    self.filtered_skills = apply_sort(filtered, self.sort_mode);
}
```

### Sort Logic

```rust
fn apply_sort(skills: Vec<(String, Skill)>, mode: SortMode) -> Vec<(String, Skill)> {
    match mode {
        SortMode::StarsDesc => skills.sort_by(|a, b| b.stars.cmp(&a.stars)),
        SortMode::StarsAsc => skills.sort_by(|a, b| a.stars.cmp(&b.stars)),
        SortMode::ScoreDesc => skills.sort_by(|a, b| b.score.cmp(&a.score)),
        SortMode::ScoreAsc => skills.sort_by(|a, b| a.score.cmp(&b.score)),
        SortMode::NameAsc => skills.sort_by(|a, b| a.name.cmp(&b.name)),
        SortMode::NameDesc => skills.sort_by(|a, b| b.name.cmp(&a.name)),
    }
    skills
}
```

## File Changes

### Files to Modify

- `src/tui/skill_selector.rs` - Complete rewrite with new state machine

### Files to Create

- `src/tui/domain_tabs.rs` - Domain tab widget
- `src/tui/tag_popup.rs` - Tag selection popup
- `src/tui/status_bar.rs` - Status bar widget
- `src/tui/help_bar.rs` - Help bar widget

### Files to Update

- `src/tui/mod.rs` - Export new widgets
- `src/cli/init.rs` - Remove project_tags filtering logic

## Implementation Order

1. Create state enums (`SortMode`, `DomainTab`)
2. Build `SkillSelectorState` struct with filter/sort methods
3. Implement domain tabs widget
4. Implement status bar widget
5. Implement help bar widget
6. Implement tag popup
7. Wire up key event handlers
8. Update `init.rs` to pass all skills (remove auto-filter)
9. Test filtering + sorting combinations

## Success Criteria

- All skills shown by default
- Domain tabs filter correctly
- Tag popup filters with AND logic
- Sorting cycles through all 6 modes
- Status bar reflects current state
- Selection state preserved across filter/sort changes
- Esc cancels cleanly