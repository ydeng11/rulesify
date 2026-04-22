# Rebuild Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rewrite rulesify as an agent/skill discovery tool with registry, project scanning, and interactive TUI setup.

**Architecture:** Layered modules (cli, registry, scanner, tui, installer, models, utils). Built-in TOML registry with runtime fetch fallback. Ratatui-based interactive selection.

**Tech Stack:** Rust + clap + tokio + reqwest + ratatui + crossterm + serde + toml

---

## Phase 1: Foundation (Models + Utils + Registry)

### Task 1: Delete Legacy Code

**Files:**
- Delete: `src/` (entire directory)
- Delete: `Cargo.toml` (will rewrite)

**Step 1: Delete src directory**

Run: `rm -rf src/`
Expected: src/ removed

**Step 2: Delete old Cargo.toml**

Run: `rm Cargo.toml`
Expected: Cargo.toml removed

**Step 3: Commit deletion**

```bash
git add -A
git commit -m "chore: remove legacy codebase for rebuild"
```

---

### Task 2: Create New Cargo.toml

**Files:**
- Create: `Cargo.toml`

**Step 1: Write new Cargo.toml**

```toml
[package]
name = "rulesify"
version = "0.4.0"
edition = "2021"
description = "Discover and install AI agent skills"
license = "MIT"

[dependencies]
anyhow = "1.0"
thiserror = "1.0"
clap = { version = "4.5", features = ["derive"] }
tokio = { version = "1.37", features = ["rt-multi-thread", "macros"] }
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
ratatui = "0.26"
crossterm = "0.27"
dirs = "5.0"
walkdir = "2.4"
env_logger = "0.10"
log = "0.4"

[dev-dependencies]
tempfile = "3.8"
```

**Step 2: Commit**

```bash
git add Cargo.toml
git commit -m "chore: create new Cargo.toml for rebuild"
```

---

### Task 3: Create Error Module

**Files:**
- Create: `src/utils/mod.rs`
- Create: `src/utils/error.rs`

**Step 1: Create utils module**

```rust
// src/utils/mod.rs
pub mod error;

pub use error::{Result, RulesifyError};
```

**Step 2: Create error types**

```rust
// src/utils/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RulesifyError {
    #[error("Registry fetch failed: {0}")]
    RegistryFetch(String),
    
    #[error("Skill not found: {0}")]
    SkillNotFound(String),
    
    #[error("No skills match the current filters")]
    NoMatchingSkills,
    
    #[error("Project scan failed: {0}")]
    ScanFailed(String),
    
    #[error("Config file error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),
    
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
}

pub type Result<T> = anyhow::Result<T>;
```

**Step 3: Commit**

```bash
git add src/utils/
git commit -m "feat: add error handling module"
```

---

### Task 4: Create Models - Skill

**Files:**
- Create: `src/models/mod.rs`
- Create: `src/models/skill.rs`

**Step 1: Create models module**

```rust
// src/models/mod.rs
pub mod skill;
pub mod registry;
pub mod context;
pub mod config;

pub use skill::Skill;
pub use registry::Registry;
pub use context::ProjectContext;
pub use config::ProjectConfig;
```

**Step 2: Create Skill struct**

```rust
// src/models/skill.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub source: String,
    pub tags: Vec<String>,
    pub compatible_tools: Vec<String>,
    pub popularity: u32,
}

impl Skill {
    pub fn matches_tools(&self, tools: &[String]) -> bool {
        tools.iter().any(|t| self.compatible_tools.contains(t))
    }
    
    pub fn matches_tags(&self, tags: &[String]) -> bool {
        tags.iter().any(|t| self.tags.contains(t))
    }
}
```

**Step 3: Commit**

```bash
git add src/models/mod.rs src/models/skill.rs
git commit -m "feat: add Skill model"
```

---

### Task 5: Create Models - Registry

**Files:**
- Create: `src/models/registry.rs`

**Step 1: Write Registry struct**

```rust
// src/models/registry.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::models::Skill;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub updated: String,
    pub skills: HashMap<String, Skill>,
}

impl Registry {
    pub fn get_skill(&self, id: &str) -> Option<&Skill> {
        self.skills.get(id)
    }
    
    pub fn filter_by_tools(&self, tools: &[String]) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_tools(tools))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn filter_by_tags(&self, tags: &[String]) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .filter(|(_, s)| s.matches_tags(tags))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
    
    pub fn all_skills(&self) -> Vec<(String, Skill)> {
        self.skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
```

**Step 2: Commit**

```bash
git add src/models/registry.rs
git commit -m "feat: add Registry model with filtering"
```

---

### Task 6: Create Models - ProjectContext

**Files:**
- Create: `src/models/context.rs`

**Step 1: Write ProjectContext**

```rust
// src/models/context.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectContext {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub existing_tools: Vec<String>,
}

impl ProjectContext {
    pub fn to_tags(&self) -> Vec<String> {
        let mut tags = Vec::new();
        tags.extend(self.languages.iter().cloned());
        tags.extend(self.frameworks.iter().cloned());
        tags
    }
    
    pub fn has_tool(&self, tool: &str) -> bool {
        self.existing_tools.contains(&tool.to_string())
    }
}
```

**Step 2: Commit**

```bash
git add src/models/context.rs
git commit -m "feat: add ProjectContext model"
```

---

### Task 7: Create Models - ProjectConfig

**Files:**
- Create: `src/models/config.rs`

**Step 1: Write ProjectConfig**

```rust
// src/models/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub added: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub version: u32,
    pub tools: Vec<String>,
    pub installed_skills: HashMap<String, InstalledSkill>,
}

impl ProjectConfig {
    pub fn new() -> Self {
        Self {
            version: 1,
            tools: Vec::new(),
            installed_skills: HashMap::new(),
        }
    }
    
    pub fn add_skill(&mut self, id: &str, source: &str) {
        self.installed_skills.insert(
            id.to_string(),
            InstalledSkill {
                added: chrono::Local::now().format("%Y-%m-%d").to_string(),
                source: source.to_string(),
            },
        );
    }
    
    pub fn remove_skill(&mut self, id: &str) -> Option<InstalledSkill> {
        self.installed_skills.remove(id)
    }
    
    pub fn list_skills(&self) -> Vec<(String, InstalledSkill)> {
        self.installed_skills
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}
```

**Step 2: Commit**

```bash
git add src/models/config.rs
git commit -m "feat: add ProjectConfig for tracking installed skills"
```

---

### Task 8: Create Built-in Registry

**Files:**
- Create: `registry.toml` (in project root)
- Create: `src/registry/mod.rs`
- Create: `src/registry/data.rs`

**Step 1: Create registry.toml**

```toml
version = 1
updated = "2026-04-08"

[skills.test-driven-development]
name = "Test-Driven Development"
description = "Write tests before implementation code using TDD methodology"
source = "https://github.com/anthropic/skills/tree/main/tdd"
tags = ["testing", "development", "best-practices"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]
popularity = 150

[skills.systematic-debugging]
name = "Systematic Debugging"
description = "Investigate bugs using scientific method before proposing fixes"
source = "https://github.com/anthropic/skills/tree/main/debugging"
tags = ["debugging", "troubleshooting", "investigation"]
compatible_tools = ["cursor", "claude-code", "cline"]
popularity = 89

[skills.brainstorming]
name = "Brainstorming"
description = "Explore user intent and design before implementation"
source = "https://github.com/anthropic/skills/tree/main/brainstorming"
tags = ["design", "planning", "creative"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]
popularity = 120

[skills.verification-before-completion]
name = "Verification Before Completion"
description = "Run verification commands before claiming work is complete"
source = "https://github.com/anthropic/skills/tree/main/verification"
tags = ["quality", "verification", "testing"]
compatible_tools = ["cursor", "claude-code"]
popularity = 95

[skills.writing-plans]
name = "Writing Plans"
description = "Create implementation plans before touching code"
source = "https://github.com/anthropic/skills/tree/main/planning"
tags = ["planning", "documentation", "process"]
compatible_tools = ["cursor", "claude-code", "cline", "goose"]
popularity = 110
```

**Step 2: Create registry module**

```rust
// src/registry/mod.rs
pub mod data;
pub mod fetch;
pub mod cache;

pub use data::load_builtin;
pub use fetch::fetch_registry;
pub use cache::RegistryCache;
```

**Step 3: Create data.rs with include_str**

```rust
// src/registry/data.rs
use crate::models::Registry;
use crate::utils::Result;

pub fn load_builtin() -> Result<Registry> {
    let content = include_str!("../../registry.toml");
    let registry: Registry = toml::from_str(content)?;
    Ok(registry)
}
```

**Step 4: Commit**

```bash
git add registry.toml src/registry/
git commit -m "feat: add built-in registry with skill metadata"
```

---

### Task 9: Create Registry Fetcher

**Files:**
- Create: `src/registry/fetch.rs`

**Step 1: Write fetch module**

```rust
// src/registry/fetch.rs
use crate::models::Registry;
use crate::utils::{Result, RulesifyError};

const REGISTRY_URL: &str = "https://raw.githubusercontent.com/user/rulesify/main/registry.toml";

pub async fn fetch_registry() -> Result<Registry> {
    let client = reqwest::Client::new();
    let response = client
        .get(REGISTRY_URL)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| RulesifyError::RegistryFetch(e.to_string()))?;
    
    if !response.status().is_success() {
        return Err(RulesifyError::RegistryFetch(
            format!("HTTP {}", response.status())
        ).into());
    }
    
    let content = response.text().await?;
    let registry: Registry = toml::from_str(&content)?;
    Ok(registry)
}
```

**Step 2: Commit**

```bash
git add src/registry/fetch.rs
git commit -m "feat: add registry fetcher from GitHub"
```

---

### Task 10: Create Registry Cache

**Files:**
- Create: `src/registry/cache.rs`

**Step 1: Write cache module**

```rust
// src/registry/cache.rs
use crate::models::Registry;
use crate::utils::{Result, RulesifyError};
use std::path::PathBuf;
use std::fs;

pub struct RegistryCache {
    cache_path: PathBuf,
}

impl RegistryCache {
    pub fn new() -> Self {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rulesify");
        
        Self {
            cache_path: cache_dir.join("registry.toml"),
        }
    }
    
    pub fn load(&self) -> Result<Option<Registry>> {
        if !self.cache_path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&self.cache_path)?;
        let registry: Registry = toml::from_str(&content)?;
        Ok(Some(registry))
    }
    
    pub fn save(&self, registry: &Registry) -> Result<()> {
        let parent = self.cache_path.parent().unwrap();
        fs::create_dir_all(parent)?;
        
        let content = toml::to_string_pretty(registry)?;
        fs::write(&self.cache_path, content)?;
        Ok(())
    }
    
    pub fn clear(&self) -> Result<()> {
        if self.cache_path.exists() {
            fs::remove_file(&self.cache_path)?;
        }
        Ok(())
    }
}
```

**Step 2: Commit**

```bash
git add src/registry/cache.rs
git commit -m "feat: add registry cache with load/save/clear"
```

---

## Phase 2: Scanner

### Task 11: Create Scanner Module

**Files:**
- Create: `src/scanner/mod.rs`
- Create: `src/scanner/language.rs`
- Create: `src/scanner/framework.rs`
- Create: `src/scanner/tool_config.rs`

**Step 1: Create scanner module**

```rust
// src/scanner/mod.rs
pub mod language;
pub mod framework;
pub mod tool_config;

use crate::models::ProjectContext;
use crate::utils::Result;

pub fn scan_project(path: &std::path::Path) -> Result<ProjectContext> {
    let languages = language::detect(path)?;
    let frameworks = framework::detect(path)?;
    let existing_tools = tool_config::detect(path)?;
    
    Ok(ProjectContext {
        languages,
        frameworks,
        existing_tools,
    })
}
```

**Step 2: Commit**

```bash
git add src/scanner/mod.rs
git commit -m "feat: add scanner module skeleton"
```

---

### Task 12: Create Language Detector

**Files:**
- Create: `src/scanner/language.rs`

**Step 1: Write language detector**

```rust
// src/scanner/language.rs
use crate::utils::Result;
use std::path::Path;
use walkdir::WalkDir;
use std::collections::HashSet;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut languages = HashSet::new();
    
    for entry in WalkDir::new(path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let ext = entry.path()
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match ext {
            "rs" => { languages.insert("rust"); },
            "ts" | "tsx" => { languages.insert("typescript"); },
            "js" | "jsx" => { languages.insert("javascript"); },
            "py" => { languages.insert("python"); },
            "go" => { languages.insert("go"); },
            "java" => { languages.insert("java"); },
            "rb" => { languages.insert("ruby"); },
            "php" => { languages.insert("php"); },
            "c" | "cpp" | "cc" => { languages.insert("cpp"); },
            _ => {}
        }
    }
    
    // Also check config files
    if path.join("Cargo.toml").exists() {
        languages.insert("rust");
    }
    if path.join("package.json").exists() {
        if !languages.contains("typescript") {
            languages.insert("javascript");
        }
    }
    if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
        languages.insert("python");
    }
    if path.join("go.mod").exists() {
        languages.insert("go");
    }
    
    Ok(languages.into_iter().map(|s| s.to_string()).collect())
}
```

**Step 2: Commit**

```bash
git add src/scanner/language.rs
git commit -m "feat: add language detection from file extensions and configs"
```

---

### Task 13: Create Framework Detector

**Files:**
- Create: `src/scanner/framework.rs`

**Step 1: Write framework detector**

```rust
// src/scanner/framework.rs
use crate::utils::Result;
use std::path::Path;
use std::collections::HashSet;
use std::fs;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut frameworks = HashSet::new();
    
    // Rust frameworks from Cargo.toml
    if let Ok(content) = fs::read_to_string(path.join("Cargo.toml")) {
        if content.contains("tokio") { frameworks.insert("tokio"); }
        if content.contains("actix") { frameworks.insert("actix"); }
        if content.contains("serde") { frameworks.insert("serde"); }
    }
    
    // JS/TS frameworks from package.json
    if let Ok(content) = fs::read_to_string(path.join("package.json")) {
        if content.contains("\"react\"") { frameworks.insert("react"); }
        if content.contains("\"next\"") { frameworks.insert("nextjs"); }
        if content.contains("\"vue\"") { frameworks.insert("vue"); }
        if content.contains("\"svelte\"") { frameworks.insert("svelte"); }
        if content.contains("\"express\"") { frameworks.insert("express"); }
        if content.contains("\"nestjs\"") { frameworks.insert("nestjs"); }
    }
    
    // Python frameworks
    if path.join("pyproject.toml").exists() {
        if let Ok(content) = fs::read_to_string(path.join("pyproject.toml")) {
            if content.contains("django") { frameworks.insert("django"); }
            if content.contains("flask") { frameworks.insert("flask"); }
            if content.contains("fastapi") { frameworks.insert("fastapi"); }
        }
    }
    
    Ok(frameworks.into_iter().map(|s| s.to_string()).collect())
}
```

**Step 2: Commit**

```bash
git add src/scanner/framework.rs
git commit -m "feat: add framework detection from package files"
```

---

### Task 14: Create Tool Config Detector

**Files:**
- Create: `src/scanner/tool_config.rs`

**Step 1: Write tool config detector**

```rust
// src/scanner/tool_config.rs
use crate::utils::Result;
use std::path::Path;
use std::collections::HashSet;

pub fn detect(path: &Path) -> Result<Vec<String>> {
    let mut tools = HashSet::new();
    
    // Cursor
    if path.join(".cursor").exists() || path.join(".cursorrules").exists() {
        tools.insert("cursor");
    }
    
    // Claude Code
    if path.join("CLAUDE.md").exists() {
        tools.insert("claude-code");
    }
    
    // Cline
    if path.join(".clinerules").exists() {
        tools.insert("cline");
    }
    
    // Goose
    if path.join(".goosehints").exists() {
        tools.insert("goose");
    }
    
    Ok(tools.into_iter().map(|s| s.to_string()).collect())
}
```

**Step 2: Commit**

```bash
git add src/scanner/tool_config.rs
git commit -m "feat: add AI tool config detection"
```

---

## Phase 3: Installer

### Task 15: Create Installer Module

**Files:**
- Create: `src/installer/mod.rs`
- Create: `src/installer/instructions.rs`

**Step 1: Create installer module**

```rust
// src/installer/mod.rs
pub mod instructions;

pub use instructions::generate_instructions;
```

**Step 2: Create instructions generator**

```rust
// src/installer/instructions.rs
use crate::models::Skill;

pub fn generate_instructions(skills: &[(String, Skill)], tools: &[String]) -> String {
    let mut output = String::new();
    
    output.push_str("# Installation Instructions\n\n");
    output.push_str("Copy the following instructions for your AI agent:\n\n");
    output.push_str("---\n\n");
    
    for (id, skill) in skills {
        output.push_str(&format!("## Skill: {}\n\n", skill.name));
        output.push_str(&format!("**Source:** {}\n\n", skill.source));
        
        for tool in tools {
            if skill.compatible_tools.contains(tool) {
                output.push_str(&format!("### For {}\n\n", tool));
                output.push_str(&generate_tool_specific_instructions(tool, &skill.source));
            }
        }
        
        output.push_str("\n---\n\n");
    }
    
    output
}

fn generate_tool_specific_instructions(tool: &str, source: &str) -> String {
    match tool {
        "cursor" => format!(
            "1. Fetch the skill instructions from: {}\n2. Create `.cursor/rules/<skill-name>.md` with the content\n",
            source
        ),
        "claude-code" => format!(
            "1. Fetch the skill instructions from: {}\n2. Append to `CLAUDE.md` or create a dedicated section\n",
            source
        ),
        "cline" => format!(
            "1. Fetch the skill instructions from: {}\n2. Add to `.clinerules` file\n",
            source
        ),
        "goose" => format!(
            "1. Fetch the skill instructions from: {}\n2. Add to `.goosehints` file\n",
            source
        ),
        _ => format!("Install from: {}\n", source),
    }
}
```

**Step 3: Commit**

```bash
git add src/installer/
git commit -m "feat: add instruction generator for skill installation"
```

---

## Phase 4: TUI

### Task 16: Create TUI Module

**Files:**
- Create: `src/tui/mod.rs`
- Create: `src/tui/tool_picker.rs`
- Create: `src/tui/skill_selector.rs`

**Step 1: Create TUI module**

```rust
// src/tui/mod.rs
pub mod tool_picker;
pub mod skill_selector;

pub use tool_picker::ToolPicker;
pub use skill_selector::SkillSelector;
```

**Step 2: Commit**

```bash
git add src/tui/mod.rs
git commit -m "feat: add TUI module skeleton"
```

---

### Task 17: Create Tool Picker

**Files:**
- Create: `src/tui/tool_picker.rs`

**Step 1: Write tool picker (simplified for plan)**

```rust
// src/tui/tool_picker.rs
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    widgets::{Block, Borders, Checkbox, List, ListItem},
    Frame, Terminal,
};
use std::io;

const TOOLS: [&str; 4] = ["cursor", "claude-code", "cline", "goose"];

pub struct ToolPicker {
    selected: Vec<bool>,
}

impl ToolPicker {
    pub fn new() -> Self {
        Self {
            selected: vec![false, false, false, false],
        }
    }
    
    pub fn run() -> io::Result<Vec<String>> {
        // Setup terminal
        let mut terminal = Terminal::new(crossterm::Terminal::new(io::stdout()))?;
        terminal.clear()?;
        
        let mut picker = Self::new();
        
        loop {
            terminal.draw(|f| picker.render(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => picker.selected[0] = !picker.selected[0],
                    KeyCode::Char('2') => picker.selected[1] = !picker.selected[1],
                    KeyCode::Char('3') => picker.selected[2] = !picker.selected[2],
                    KeyCode::Char('4') => picker.selected[3] = !picker.selected[3],
                    KeyCode::Enter => break,
                    KeyCode::Esc => return Ok(vec![]),
                    _ => {}
                }
            }
        }
        
        // Restore terminal
        terminal.clear()?;
        
        Ok(TOOLS.iter()
            .zip(picker.selected.iter())
            .filter_map(|(t, s)| if *s { Some(t.to_string()) } else { None })
            .collect())
    }
    
    fn render(&self, f: &mut Frame) {
        // Render UI (simplified)
        let items: Vec<ListItem> = TOOLS.iter()
            .zip(self.selected.iter())
            .map(|(t, s)| {
                let symbol = if *s { "[x]" } else { "[ ]" };
                ListItem::new(format!("{}. {} {}", 
                    TOOLS.iter().position(|x| x == t).unwrap() + 1,
                    symbol,
                    t
                ))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title("Select AI Tools (1-4 to toggle, Enter to confirm)")
                .borders(Borders::ALL));
        
        f.render_widget(list, f.size());
    }
}
```

**Step 2: Commit**

```bash
git add src/tui/tool_picker.rs
git commit -m "feat: add tool picker TUI component"
```

---

### Task 18: Create Skill Selector

**Files:**
- Create: `src/tui/skill_selector.rs`

**Step 1: Write skill selector**

```rust
// src/tui/skill_selector.rs
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use crate::models::Skill;
use std::io;

pub struct SkillSelector {
    skills: Vec<(String, Skill)>,
    selected_indices: Vec<usize>,
    current_index: usize,
}

impl SkillSelector {
    pub fn new(skills: Vec<(String, Skill)>) -> Self {
        Self {
            skills,
            selected_indices: vec![],
            current_index: 0,
        }
    }
    
    pub fn run() -> io::Result<Vec<(String, Skill)>> {
        let mut terminal = Terminal::new(crossterm::Terminal::new(io::stdout()))?;
        
        // Implementation would include full TUI loop
        // For plan: simplified version
        
        Ok(vec![])
    }
    
    fn render(&self, f: &mut Frame) {
        let items: Vec<ListItem> = self.skills.iter()
            .enumerate()
            .map(|(i, (id, skill))| {
                let marker = if self.selected_indices.contains(&i) { "[x]" } else { "[ ]" };
                let cursor = if i == self.current_index { ">" } else { " " };
                ListItem::new(format!(
                    "{}{} {} - {} (★{})",
                    cursor, marker, skill.name, skill.description, skill.popularity
                ))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title("Select Skills (↑↓ navigate, Space select, Enter confirm)")
                .borders(Borders::ALL));
        
        f.render_widget(list, f.size());
    }
}
```

**Step 2: Commit**

```bash
git add src/tui/skill_selector.rs
git commit -m "feat: add skill selector TUI component"
```

---

## Phase 5: CLI

### Task 19: Create CLI Module

**Files:**
- Create: `src/cli/mod.rs`
- Create: `src/cli/init.rs`
- Create: `src/cli/skill.rs`

**Step 1: Create CLI module**

```rust
// src/cli/mod.rs
pub mod init;
pub mod skill;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rulesify")]
#[command(about = "Discover and install AI agent skills")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Interactive setup to discover and install skills
    Init,
    
    /// Manage installed skills
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
}

#[derive(Subcommand)]
enum SkillCommands {
    /// List installed skills
    List,
    
    /// Add a skill from registry
    Add {
        /// Skill ID to add
        id: String,
    },
    
    /// Remove an installed skill
    Remove {
        /// Skill ID to remove
        id: String,
    },
    
    /// Update registry cache
    Update,
}

pub async fn run(cli: Cli) -> crate::utils::Result<()> {
    match cli.command {
        Commands::Init => init::run(cli.verbose).await?,
        Commands::Skill { command } => skill::run(command, cli.verbose).await?,
    }
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/cli/mod.rs
git commit -m "feat: add CLI structure with init and skill commands"
```

---

### Task 20: Create Init Command

**Files:**
- Create: `src/cli/init.rs`

**Step 1: Write init command**

```rust
// src/cli/init.rs
use crate::registry::{load_builtin, fetch_registry, RegistryCache};
use crate::scanner::scan_project;
use crate::tui::{ToolPicker, SkillSelector};
use crate::installer::generate_instructions;
use crate::models::{ProjectConfig, Registry};
use crate::utils::Result;
use std::path::Path;

pub async fn run(verbose: bool) -> Result<()> {
    let project_path = Path::new(".");
    
    // 1. Scan project
    if verbose {
        println!("Scanning project...");
    }
    let context = scan_project(project_path)?;
    
    if verbose {
        println!("Languages: {:?}", context.languages);
        println!("Frameworks: {:?}", context.frameworks);
        println!("Existing tools: {:?}", context.existing_tools);
    }
    
    // 2. Pick tools
    println!("Select AI tools you use:");
    let tools = ToolPicker::run()?;
    
    if tools.is_empty() {
        println!("No tools selected. Exiting.");
        return Ok(());
    }
    
    // 3. Load registry
    let registry = load_registry().await?;
    
    // 4. Filter and match skills
    let project_tags = context.to_tags();
    let matching_skills = registry.skills.iter()
        .filter(|(_, s)| s.matches_tools(&tools))
        .filter(|(_, s)| project_tags.is_empty() || s.matches_tags(&project_tags))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<Vec<_>>();
    
    if matching_skills.is_empty() {
        println!("No skills match your project context. Try broader filters.");
        return Ok(());
    }
    
    // 5. Select skills
    println!("\nSelect skills to install:");
    let selected = SkillSelector::new(matching_skills).run()?;
    
    if selected.is_empty() {
        println!("No skills selected. Exiting.");
        return Ok(());
    }
    
    // 6. Generate instructions
    let instructions = generate_instructions(&selected, &tools);
    println!("\n{}", instructions);
    
    // 7. Save config
    let mut config = ProjectConfig::new();
    config.tools = tools;
    for (id, skill) in &selected {
        config.add_skill(id, &skill.source);
    }
    
    std::fs::write(".rulesify.toml", toml::to_string_pretty(&config)?)?;
    println!("\nSaved configuration to .rulesify.toml");
    
    Ok(())
}

async fn load_registry() -> Result<Registry> {
    let cache = RegistryCache::new();
    
    // Try fetch first
    if let Ok(registry) = fetch_registry().await {
        cache.save(&registry)?;
        return Ok(registry);
    }
    
    // Fallback to cache
    if let Some(registry) = cache.load()? {
        return Ok(registry);
    }
    
    // Fallback to builtin
    load_builtin()
}
```

**Step 2: Commit**

```bash
git add src/cli/init.rs
git commit -m "feat: add init command with full flow"
```

---

### Task 21: Create Skill Command

**Files:**
- Create: `src/cli/skill.rs`

**Step 1: Write skill command**

```rust
// src/cli/skill.rs
use crate::cli::{SkillCommands};
use crate::registry::{load_builtin, fetch_registry, RegistryCache};
use crate::models::{ProjectConfig, Registry};
use crate::utils::{Result, RulesifyError};
use std::path::Path;

pub async fn run(command: SkillCommands, verbose: bool) -> Result<()> {
    match command {
        SkillCommands::List => list_skills(verbose),
        SkillCommands::Add { id } => add_skill(id, verbose).await?,
        SkillCommands::Remove { id } => remove_skill(id, verbose),
        SkillCommands::Update => update_registry(verbose).await?,
    }
    Ok(())
}

fn list_skills(verbose: bool) -> Result<()> {
    let config_path = Path::new(".rulesify.toml");
    
    if !config_path.exists() {
        println!("No skills installed. Run `rulesify init` first.");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(config_path)?;
    let config: ProjectConfig = toml::from_str(&content)?;
    
    println!("Installed skills:");
    for (id, info) in config.list_skills() {
        println!("  - {} (added: {})", id, info.added);
        if verbose {
            println!("    Source: {}", info.source);
        }
    }
    
    Ok(())
}

async fn add_skill(id: String, verbose: bool) -> Result<()> {
    let registry = load_registry().await?;
    
    let skill = registry.get_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;
    
    let config_path = Path::new(".rulesify.toml");
    let mut config = if config_path.exists() {
        let content = std::fs::read_to_string(config_path)?;
        toml::from_str::<ProjectConfig>(&content)?
    } else {
        ProjectConfig::new()
    };
    
    config.add_skill(&id, &skill.source);
    
    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;
    
    println!("Added skill: {}", skill.name);
    println!("Source: {}", skill.source);
    println!("\nInstall instructions:");
    println!("  Fetch from {} and add to your AI tool config", skill.source);
    
    Ok(())
}

fn remove_skill(id: String, verbose: bool) -> Result<()> {
    let config_path = Path::new(".rulesify.toml");
    
    if !config_path.exists() {
        println!("No skills installed.");
        return Ok(());
    }
    
    let content = std::fs::read_to_string(config_path)?;
    let mut config: ProjectConfig = toml::from_str(&content)?;
    
    let removed = config.remove_skill(&id)
        .ok_or_else(|| RulesifyError::SkillNotFound(id.clone()))?;
    
    std::fs::write(config_path, toml::to_string_pretty(&config)?)?;
    
    println!("Removed skill: {}", id);
    println!("Added on: {}", removed.added);
    println!("\nCleanup instructions:");
    println!("  Remove skill content from your AI tool config files");
    
    Ok(())
}

async fn update_registry(verbose: bool) -> Result<()> {
    println!("Updating registry cache...");
    
    let registry = fetch_registry().await?;
    let cache = RegistryCache::new();
    cache.save(&registry)?;
    
    println!("Registry updated ({} skills)", registry.skills.len());
    
    if verbose {
        println!("Updated date: {}", registry.updated);
    }
    
    Ok(())
}

async fn load_registry() -> Result<Registry> {
    let cache = RegistryCache::new();
    
    if let Ok(registry) = fetch_registry().await {
        cache.save(&registry)?;
        return Ok(registry);
    }
    
    if let Some(registry) = cache.load()? {
        return Ok(registry);
    }
    
    load_builtin()
}
```

**Step 2: Commit**

```bash
git add src/cli/skill.rs
git commit -m "feat: add skill list/add/remove/update commands"
```

---

### Task 22: Create Main Entry Point

**Files:**
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Create lib.rs**

```rust
// src/lib.rs
pub mod cli;
pub mod models;
pub mod registry;
pub mod scanner;
pub mod tui;
pub mod installer;
pub mod utils;
```

**Step 2: Create main.rs**

```rust
// src/main.rs
use rulesify::cli::{Cli, run};
use clap::Parser;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let cli = Cli::parse();
    
    if let Err(e) = run(cli).await {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
```

**Step 3: Commit**

```bash
git add src/main.rs src/lib.rs
git commit -m "feat: add main entry point"
```

---

## Phase 6: Testing & Polish

### Task 23: Add Tests for Scanner

**Files:**
- Create: `src/scanner/language_tests.rs`

**Step 1: Write language detector test**

```rust
// src/scanner/language_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "").unwrap();
        
        let langs = detect(dir.path()).unwrap();
        assert!(langs.contains(&"rust".to_string()));
    }

    #[test]
    fn test_detect_typescript() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("app.ts"), "").unwrap();
        fs::write(dir.path().join("package.json"), "{}").unwrap();
        
        let langs = detect(dir.path()).unwrap();
        assert!(langs.contains(&"typescript".to_string()));
    }
}
```

**Step 2: Run tests**

Run: `cargo test scanner::language_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add src/scanner/language_tests.rs
git commit -m "test: add language detector tests"
```

---

### Task 24: Add Tests for Registry

**Files:**
- Create: `src/registry/data_tests.rs`

**Step 1: Write registry test**

```rust
// src/registry/data_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_builtin() {
        let registry = load_builtin().unwrap();
        assert!(registry.skills.len() > 0);
        assert!(registry.version == 1);
    }

    #[test]
    fn test_skill_exists() {
        let registry = load_builtin().unwrap();
        assert!(registry.get_skill("test-driven-development").is_some());
    }
}
```

**Step 2: Run tests**

Run: `cargo test registry::data_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add src/registry/data_tests.rs
git commit -m "test: add registry load tests"
```

---

### Task 25: Add Tests for Models

**Files:**
- Create: `src/models/skill_tests.rs`

**Step 1: Write skill model test**

```rust
// src/models/skill_tests.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_tools() {
        let skill = Skill {
            name: "Test".into(),
            description: "Desc".into(),
            source: "url".into(),
            tags: vec!["testing".into()],
            compatible_tools: vec!["cursor".into(), "claude-code".into()],
            popularity: 100,
        };
        
        assert!(skill.matches_tools(&["cursor".into()]));
        assert!(skill.matches_tools(&["claude-code".into()]));
        assert!(!skill.matches_tools(&["cline".into()]));
    }

    #[test]
    fn test_matches_tags() {
        let skill = Skill {
            name: "Test".into(),
            description: "Desc".into(),
            source: "url".into(),
            tags: vec!["testing".into(), "rust".into()],
            compatible_tools: vec!["cursor".into()],
            popularity: 100,
        };
        
        assert!(skill.matches_tags(&["testing".into()]));
        assert!(skill.matches_tags(&["rust".into()]));
        assert!(!skill.matches_tags(&["python".into()]));
    }
}
```

**Step 2: Run tests**

Run: `cargo test models::skill_tests`
Expected: PASS

**Step 3: Commit**

```bash
git add src/models/skill_tests.rs
git commit -m "test: add skill model tests"
```

---

### Task 26: Build and Verify

**Files:**
- None (verify build)

**Step 1: Run cargo check**

Run: `cargo check`
Expected: No errors

**Step 2: Run cargo build**

Run: `cargo build --release`
Expected: Successful build

**Step 3: Test basic commands**

Run: `./target/release/rulesify --help`
Expected: Help output shown

Run: `./target/release/rulesify skill list`
Expected: "No skills installed" message

**Step 4: Commit final**

```bash
git add -A
git commit -m "feat: complete rebuild with agent/skill discovery"
```

---

## Summary

**Tasks:** 26
**Phases:**
1. Foundation (Tasks 1-10): Delete legacy, create models, registry
2. Scanner (Tasks 11-14): Language, framework, tool detection
3. Installer (Task 15): Instruction generation
4. TUI (Tasks 16-18): Tool picker, skill selector
5. CLI (Tasks 19-22): Commands and main entry
6. Testing (Tasks 23-26): Tests and build verification

**Estimated Time:** 4-6 hours for execution

---
*Plan created: 2026-04-08*