use anyhow::Result;

pub fn get_default_skeleton() -> &'static str {
    r#"# -------------------------------------------------------------
#  UNIVERSAL RULE FILE (URF) – SINGLE SOURCE OF TRUTH
#  Replace <placeholders> and delete comments after editing.
# -------------------------------------------------------------

id: <rule_id>              # machine-safe slug, filled automatically
version: 0.1.0             # bump when you make breaking changes

metadata:
  name: "<Human-friendly Name>"          # appears in exported Markdown H1
  description: |
    <One-sentence description that shows up in Cursor front-matter>
  tags: []                 # e.g. [react, style, hooks]
  priority: 5              # 1 (low) → 10 (high); used for ordering
  auto_apply: false        # if true, export uses alwaysApply in Cursor

content:
  - title: "Guidelines"                  # Markdown H2 in exports
    format: markdown                      # or plaintext / code
    value: |-
      • Add your first bullet here
      • Use **block-scalar** so you keep Markdown formatting

# Optional extra sections – copy / paste as needed
#  - title: "Examples"
#    format: markdown
#    value: |-
#      ```js
#      // code demo
#      ```

references: []             # optional list of @file references
conditions: []             # optional glob patterns that trigger auto-attach

# -------------------------------------------------------------------
#  Tool-specific overrides (ignored by other exporters)
# -------------------------------------------------------------------

tool_overrides:
  cursor:
    globs: []              # e.g. [src/**/*.tsx, src/**/*.jsx]
  cline: {}
  claude-code: {}
  goose: {}
"#
}

pub fn create_skeleton_for_rule(rule_name: &str) -> Result<String> {
    let skeleton = get_default_skeleton();
    let filled = skeleton.replace("<rule_id>", rule_name)
        .replace("<Human-friendly Name>", &format!("{} Rule", rule_name))
        .replace("<One-sentence description that shows up in Cursor front-matter>", 
                 &format!("Guidelines for {}", rule_name));
    Ok(filled)
} 