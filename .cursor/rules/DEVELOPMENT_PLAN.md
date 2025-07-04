# Rulesify Development Plan

A comprehensive terminal tool for managing AI coding assistant rules across different platforms.

## Project Overview

**Rulesify** is a terminal tool written in Rust designed to facilitate the unified management of rules used across different AI coding assistants (Cursor, Cline, Claude Code, and Goose). The tool addresses the challenge of maintaining consistent rules across multiple AI platforms while respecting each tool's unique format requirements.

## Core Requirements Analysis

### Rule Format Support

- **Cursor**: MDC format with YAML frontmatter, supports globs, descriptions, and file references
- **Cline**: Simple Markdown files in `.clinerules/` directories, toggleable via UI
- **Claude Code**: `CLAUDE.md` files in multiple locations (repo root, parent/child dirs, home)
- **Goose**: Simple text-based `.goosehints` files

### User Needs
1. **Format Conversion**: Transform rules between different AI tool formats
2. **Persistent Management**: Store and organize rules in a version-controlled manner
3. **Rule Reuse**: Share rules across projects and team members
4. **Project Configuration**: Apply different rule sets per project context

## Universal Rule Format (URF)

The canonical source of truth for every rule will be **one YAML file per rule**.

Key properties:
- **YAML structure** for metadata, references, tool-specific overrides.
- Long prose stored as **block-scalar Markdown strings (`|`)** – no escaping headaches, perfect fidelity for LLMs.
- **Namespaced `tool_overrides`** blocks hold settings that belong to a single AI tool, ensuring they never leak when exporting to another tool.

Example skeleton:
```yaml
id: ts-style
version: 1.0.0
metadata:
  name: TypeScript Coding Style
  description: Enforces TS formatting & lint
  tags: [typescript, style]
  priority: 5
  auto_apply: true
content:
  - title: Code Formatting
    format: markdown
    value: |
      • Indent with **2 spaces**
      • Use _single quotes_ for strings
references:
  - @CONTRIBUTING.md
tool_overrides:
  cursor:
    globs: [src/**/*.ts]
  cline:
    toggle_default: true
```

Round-trip integrity is enforced by CI: `urf → tool file → urf` must be loss-free.

## System Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Rule Source   │    │   Rulesify      │    │   AI Tools      │
│   (Universal)   │ -> │   (Converter)   │ -> │   (Specific)    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Implementation Timeline

### ✅ Phase 1: Core Infrastructure (Weeks 1-3) - COMPLETED
- ✅ Set up Rust project with Cargo and dependencies
- ✅ Implement core data structures (Rule, ProjectConfig, etc.)
- ✅ Create file-based rule store with RuleStore trait
- ✅ Basic CLI argument parsing with clap

### ✅ Phase 2: Format Converters (Weeks 4-6) - SKELETON COMPLETED
- ✅ Implement universal rule format data structures
- ✅ Create converter trait and skeleton implementations for all 4 AI tools:
  - ✅ CursorConverter - MDC format with YAML frontmatter
  - ✅ ClineConverter - Simple Markdown format
  - ✅ ClaudeCodeConverter - CLAUDE.md format
  - ✅ GooseConverter - Plain text .goosehints format
- 🚧 Add conversion validation and testing (In Progress)
- 🚧 Handle edge cases and format variations (TODO)

### ✅ Phase 3: CLI Interface (Weeks 7-8) - SKELETON COMPLETED
- ✅ Complete command structure implementation:
  - ✅ `init` - Project initialization
  - ✅ `rule new/edit/list/show/delete` - Rule management
  - ✅ `deploy` - Rule deployment with tool selection
  - ✅ `sync` - Cross-tool synchronization
  - ✅ `template` - Template management
- 🚧 Add interactive modes for rule creation (TODO)
- 🚧 Implement import/export functionality (TODO)
- 🚧 Add comprehensive error handling (TODO)

### ✅ Phase 4: Rule Skeleton (Weeks 9-10) - COMPLETED
- ✅ Built-in YAML skeleton implementation in `src/templates/builtin.rs`
- ✅ Command `rule new` structure for creating rules from skeleton
- ✅ Template engine for placeholder replacement

#### ✅ Default Skeleton YAML - IMPLEMENTED
The skeleton is embedded in the code at `src/templates/builtin.rs` and `rulesify rule new <name>` will create a new rule file with placeholders filled.

### 🚧 Phase 5: Implementation (Weeks 11-12) - IN PROGRESS
- 🚧 Add rule validation and linting
- 🚧 Implement synchronization across tools
- 🚧 Add conflict detection and resolution
- 🚧 Performance optimization and testing

## ✅ Project Structure - COMPLETED

The project now follows the complete structure from the detailed plan:

```
rulesify/
├── Cargo.toml                    ✅ Package config with dependencies
├── README.md                     ✅ Project documentation
├── src/
│   ├── main.rs                   ✅ Binary entry point
│   ├── lib.rs                    ✅ Library exports
│   ├── cli/                      ✅ CLI interface
│   │   ├── mod.rs                ✅ Main CLI structure
│   │   └── commands/             ✅ All command implementations
│   ├── models/                   ✅ Core data structures
│   ├── store/                    ✅ Rule storage abstraction
│   ├── converters/               ✅ AI tool format converters
│   ├── templates/                ✅ Template management
│   ├── validation/               ✅ Rule validation framework
│   ├── sync/                     ✅ Cross-tool synchronization
│   └── utils/                    ✅ Utilities and configuration
├── templates/                    ✅ Rule template files
├── tests/                        ✅ Unit and integration tests
└── docs/                         ✅ Documentation and examples
```

## MVP Feature Matrix

| Capability | CLI Command(s) | Status |
|------------|----------------|--------|
| **Create/ Edit URF rule** | `rulesify rule new <name>`<br/>`rulesify rule edit <name>` | ✅ Structure implemented |
| **CRUD & List (regex)** | `rulesify rule list [-r <regex>]`<br/>`rule show` `rule delete` | ✅ Structure implemented |
| **Validate** | `rulesify validate <name> \| --all` | ✅ Framework implemented |
| **Merge & Export to tool** | `rulesify deploy --tool cursor --rules a,b [--dry-run]` | ✅ Structure implemented |
| **Conflict warning** | Auto-triggered during `deploy` | 🚧 TODO |
| **Import local tool file → URF** | `rulesify import --tool cursor <file>` | 🚧 TODO |
| **Config management** | `rulesify config edit` | ✅ Structure implemented |

## ✅ Current Status

The project has successfully completed the foundational phases:

1. **✅ Project Skeleton**: Complete Rust project structure with all modules
2. **✅ CLI Framework**: Full command structure with clap integration
3. **✅ Data Models**: Universal Rule Format and all supporting structures
4. **✅ Storage Layer**: File-based and memory storage implementations
5. **✅ Converter Framework**: Trait-based converter system for all 4 AI tools
6. **✅ Template System**: Built-in skeleton for rule creation
7. **✅ Validation Framework**: Extensible validation system
8. **✅ Testing Structure**: Unit test framework and examples

## Next Steps

1. **Implement Core Logic**: Fill in the TODO sections in command implementations
2. **Add Parsing Logic**: Implement the `convert_from_tool_format` methods
3. **Rule Validation**: Complete the validation rules and error handling
4. **File Operations**: Implement actual file reading/writing for rule management
5. **Configuration Management**: Complete the config loading/saving logic
6. **Integration Testing**: Add comprehensive tests for all converters
7. **Error Handling**: Improve error messages and edge case handling

The project is now ready for feature implementation with a solid architectural foundation in place. 