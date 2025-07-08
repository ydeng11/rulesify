use anyhow::Result;

pub fn get_default_skeleton() -> &'static str {
    r#"# =====================================================================
#  UNIVERSAL RULE FILE (URF) – SINGLE SOURCE OF TRUTH
#
#  This file defines a rule that can be deployed to all AI coding tools:
#  - Cursor (.cursor/rules/*.mdc)
#  - Cline (.clinerules/*.md)
#  - Claude Code (CLAUDE.md)
#  - Goose (.goosehints)
#
#  INSTRUCTIONS:
#  1. Replace all <placeholders> with your content
#  2. Delete these instruction comments when done
#  3. Save and run: rulesify deploy --all
# =====================================================================

# Unique identifier for this rule (auto-filled from filename)
id: <rule_id>

# Semantic version - increment when making breaking changes
version: 0.1.0

# Core metadata that appears in all exported formats
metadata:
  # Human-readable name - becomes H1 heading in exported files
  name: "<Human-friendly Name>"

  # Brief description - shows up in Cursor frontmatter and tool descriptions
  description: |
    <One-sentence description that shows up in Cursor front-matter>

  # Tags for categorization and search (e.g. [react, typescript, testing])
  tags: []

  # Priority for rule ordering: 1 (low) → 10 (high)
  priority: 5

# Content sections - these become H2 headings in exported files
content:
  # Main guidelines section
  - title: "Guidelines"
    format: markdown        # Options: markdown, plaintext, code
    value: |-
      • Add your first bullet here
      • Use **block-scalar** (|-) to preserve Markdown formatting
      • Each bullet should be a specific, actionable instruction
      • Example: "Always use TypeScript interfaces for props"

# OPTIONAL: Add more content sections by copying the pattern above
#  - title: "Examples"
#    format: markdown
#    value: |-
#      ```typescript
#      // Good: Use descriptive names
#      interface UserProps {
#        name: string;
#        email: string;
#      }
#
#      // Bad: Vague names
#      interface Props {
#        a: string;
#        b: string;
#      }
#      ```

# OPTIONAL: Reference external files (appears as @filename in Cursor)
references: []             # Example: [@README.md, @docs/style-guide.md]

# OPTIONAL: File patterns that trigger auto-attachment
conditions: []             # Example: [type: file_pattern, value: "src/**/*.ts"]

# =====================================================================
#  TOOL-SPECIFIC OVERRIDES
#  These settings only apply to specific tools and are ignored by others
# =====================================================================

tool_overrides:
  # Cursor-specific settings
  cursor:
    # Application mode - how Cursor should apply this rule:
    # • always: Apply to every chat and cmd-k session (equivalent to old auto_apply: true)
    # • intelligent: When Agent decides it's relevant based on description (RECOMMENDED)
    # • specific_files: When file matches specified patterns (uses globs below)
    # • manual: Only when @-mentioned by user (equivalent to old auto_apply: false)
    apply_mode: intelligent     # Options: always | intelligent | specific_files | manual

    globs: []                   # File patterns: [src/**/*.tsx, src/**/*.jsx]
                               # Only used when apply_mode is "specific_files"

    # Legacy field for backwards compatibility (deprecated - use apply_mode instead)
    # auto_apply: false

  # Cline-specific settings
  cline: {}

  # Claude Code-specific settings
  claude-code: {}

  # Goose-specific settings
  goose: {}

# =====================================================================
#  USAGE EXAMPLES:
#
#  Deploy to all tools:     rulesify deploy --all
#  Deploy to Cursor only:   rulesify deploy --tool cursor --rule <rule_id>
#  Edit this rule:          rulesify rule edit <rule_id>
#  View deployed rules:     rulesify rule list
# =====================================================================
"#
}

/// Creates a skeleton rule with both the sanitized rule ID and human-friendly name
pub fn create_skeleton_for_rule(rule_id: &str) -> Result<String> {
    let skeleton = get_default_skeleton();
    let filled = skeleton
        .replace("<rule_id>", rule_id)
        .replace("<Human-friendly Name>", &format!("{} Rule", rule_id))
        .replace(
            "<One-sentence description that shows up in Cursor front-matter>",
            &format!("Guidelines for {}", rule_id),
        );
    Ok(filled)
}

/// Creates a skeleton rule with custom human-friendly name based on original user input
pub fn create_skeleton_for_rule_with_display_name(
    rule_id: &str,
    original_name: &str,
) -> Result<String> {
    let skeleton = get_default_skeleton();
    let filled = skeleton
        .replace("<rule_id>", rule_id)
        .replace("<Human-friendly Name>", &format!("{} Rule", original_name))
        .replace(
            "<One-sentence description that shows up in Cursor front-matter>",
            &format!("Guidelines for {}", original_name),
        );
    Ok(filled)
}
