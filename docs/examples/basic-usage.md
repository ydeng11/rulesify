# Basic Usage Examples

## Creating a New Rule

```bash
# Create a new rule from the default skeleton
rulesify rule new my-coding-standards

# Edit the rule (opens in $EDITOR)
rulesify rule edit my-coding-standards
```

## Deploying Rules

```bash
# Deploy to Cursor
rulesify deploy --tool cursor --rules my-coding-standards

# Deploy to all enabled tools
rulesify deploy --all

# Dry run to see what would be deployed
rulesify deploy --tool cline --rules my-coding-standards --dry-run
```

## Managing Rules

```bash
# List all rules
rulesify rule list

# Search rules with regex
rulesify rule list --regex "typescript.*"

# Show rule details
rulesify rule show my-coding-standards

# Delete a rule
rulesify rule delete my-coding-standards
``` 