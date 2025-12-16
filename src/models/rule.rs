use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub content: String,
    pub tags: Vec<String>,
    pub category: RuleCategory,
    pub scope: RuleScope,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCategory {
    CodeStyle,
    Testing,
    Documentation,
    Architecture,
    Workflow,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleScope {
    Global,    // Available across all projects
    Workspace, // Available within a workspace
    Project,   // Project-specific
}

// Universal rule format for conversion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalRule {
    pub id: String,
    pub version: String,
    pub metadata: RuleMetadata,
    pub content: Vec<RuleContent>,
    pub references: Vec<FileReference>,
    pub conditions: Vec<RuleCondition>,
    pub tool_overrides: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleContent {
    pub title: String,
    pub format: ContentFormat,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentFormat {
    #[serde(rename = "markdown")]
    Markdown,
    #[serde(rename = "plaintext")]
    PlainText,
    #[serde(rename = "code")]
    Code,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "FileReferenceInput")]
pub struct FileReference {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum FileReferenceInput {
    String(String),
    Object { path: String },
}

impl From<FileReferenceInput> for FileReference {
    fn from(input: FileReferenceInput) -> Self {
        match input {
            FileReferenceInput::String(path) => FileReference { path },
            FileReferenceInput::Object { path } => FileReference { path },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum RuleCondition {
    #[serde(rename = "file_pattern")]
    FilePattern { value: String },
    #[serde(rename = "regex")]
    Regex { value: String },
}
