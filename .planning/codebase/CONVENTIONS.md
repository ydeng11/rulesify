# Coding Conventions

**Analysis Date:** 2026-04-02

## Style & Formatting

**Language:** Rust (Edition 2021)

**Formatting Tools:**
- No custom `rustfmt.toml` or `.clippy.toml` detected - uses standard Rust formatting conventions
- Default Rust style: 4-space indentation, max line length 100

**Standard Rust Style Applied:**
- Use `snake_case` for function and variable names
- Use `PascalCase` for type, struct, enum, and trait names
- Use `SCREAMING_SNAKE_CASE` for constants
- Place `pub` modifier before `fn`, `struct`, `enum`, etc.
- Opening braces on same line for function/struct definitions

## Naming Conventions

**Files:**
- Module files use `snake_case`: `file_store.rs`, `content_validator.rs`
- Test files use `snake_case` with `_tests.rs` suffix: `storage_tests.rs`, `converter_tests.rs`
- Module organizational files named `mod.rs`

**Functions:**
- `snake_case` pattern: `load_rule`, `save_rule`, `convert_to_tool_format`
- Helper functions prefixed with purpose: `get_converter`, `prompt_for_merged_rule_id`
- Test helper functions prefixed with `create_`: `create_test_rule`, `create_valid_rule`

**Variables:**
- `snake_case` for local variables: `rule_name`, `config_file`, `deployment_path`
- Short names acceptable in small scopes: `e` for errors, `msg` for messages

**Types:**
- Structs use `PascalCase`: `UniversalRule`, `FileStore`, `ContentValidator`
- Enums use `PascalCase` for type and variants: `RuleCategory`, `Severity::Error`
- Traits use `PascalCase`: `RuleStore`, `RuleConverter`, `Validator`

**Constants:**
- File extension patterns defined inline, not as constants

## Code Patterns

**Error Handling:**
- Use `anyhow::Result<T>` for fallible operations throughout
- Use `.with_context(|| format!("...", ...))` to add contextual error messages
- Use `anyhow::bail!("message")` for early error returns
- Pattern in `src/cli/commands/deploy.rs`:
```rust
let content = fs::read_to_string(&path)
    .with_context(|| format!("Failed to read rule file: {}", path.display()))?;
```

**Logging:**
- Use `log` crate with `debug`, `info`, `error` macros
- Debug logging for command execution flow: `debug!("Executing CLI command: {:?}", self.command)`
- Error logging for failures: `error!("Command execution failed: {}", e)`
- Info logging for successful operations: `info!("Successfully deployed rule '{}' to {}", ...)`
- Initialize logger in main: `env_logger::init()`

**Option/Result Handling:**
- Use `.ok_or_else(|| anyhow::anyhow!("message"))` for Option-to-Result conversion
- Use `.context()` for adding error context
- Use `?` operator for propagation
- Use `if let Err(ref e) = result` for error handling without early return

**Async Operations:**
- Tokio runtime available but most operations are synchronous
- `tokio` dependency present with `rt-multi-thread` and `macros` features

## Import Organization

**Order:**
1. External crate imports: `use anyhow::{Context, Result};`
2. Standard library imports: `use std::fs;`, `use std::path::{Path, PathBuf};`
3. Internal crate imports: `use crate::models::rule::UniversalRule;`

**Pattern in `src/cli/commands/deploy.rs`:**
```rust
use anyhow::{Context, Result};
use log::{debug, error, info};
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::converters::{...};
use crate::models::rule::{...};
use crate::store::{file_store::FileStore, RuleStore};
use crate::utils::config::load_config_from_path;
```

**Module Structure:**
- Each module has a `mod.rs` exposing public types
- Private implementations in separate files
- Re-export pattern: `pub use self::file_store::FileStore;`

## Module Design

**Exports:**
- Public items explicitly marked with `pub`
- Modules expose traits and key types through `mod.rs`
- Example from `src/store/mod.rs`:
```rust
pub mod file_store;
pub mod memory_store;

use crate::models::rule::UniversalRule;
use anyhow::Result;

pub trait RuleStore {
    fn load_rule(&self, id: &str) -> Result<Option<UniversalRule>>;
    // ...
}
```

**Trait-Based Abstraction:**
- Core abstractions defined as traits: `RuleStore`, `RuleConverter`, `Validator`
- Multiple implementations provided: `FileStore`, `MemoryStore`
- Dynamic dispatch with `Box<dyn Trait>` for flexibility

**Struct Construction:**
- Use `Self` in constructors: `pub fn new() -> Self { Self }`
- Implement `Default` trait where appropriate:
```rust
impl Default for TemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}
```

## Documentation

**Comments:**
- Section comments use `//` for brief explanations
- Doc comments (`///`) used for public APIs sparingly
- Inline comments for complex logic explanations

**CLI Documentation:**
- Comprehensive `clap` docstrings with examples in `src/cli/mod.rs`:
```rust
#[command(
    long_about = r#"Rulesify manages Universal Rule Files (URF)...
EXAMPLES:
    rulesify rule new typescript-style    # Create a new rule
    rulesify deploy --all                 # Deploy all rules to all tools
..."#
)]
```

## Function Design

**Size:**
- Functions typically 10-50 lines
- Complex operations split into helper functions
- Example: `run()` delegates to specialized helpers like `deploy_rule()`, `merge_rules()`

**Parameters:**
- Use `&str` for string parameters when possible
- Use `&Path` or `PathBuf` appropriately for paths
- Use `Option<T>` for optional parameters

**Return Values:**
- Return `Result<T>` for fallible operations
- Return `Option<T>` for lookup operations
- Use unit `()` for successful side-effect operations

## Serde Patterns

**Serialization:**
- Derive macros: `#[derive(Debug, Clone, Serialize, Deserialize)]`
- Custom renaming for enum variants:
```rust
#[serde(rename = "markdown")]
Markdown,
#[serde(rename = "plaintext")]
PlainText,
```

- Tagged enums:
```rust
#[serde(tag = "type")]
pub enum RuleCondition {
    #[serde(rename = "file_pattern")]
    FilePattern { value: String },
}
```

- Custom deserialization with `#[serde(from = "InputType")]`:
```rust
#[serde(from = "FileReferenceInput")]
pub struct FileReference { pub path: String }
```

---

*Convention analysis: 2026-04-02*