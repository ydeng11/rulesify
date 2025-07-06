use clap::{Parser, Subcommand};
use log::{debug, error};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser)]
#[command(name = "rulesify")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "A CLI tool for managing AI assistant rules")]
#[command(
    long_about = r#"Rulesify manages Universal Rule Files (URF) that can be deployed to multiple AI coding tools:
- Cursor (.cursor/rules/*.mdc)
- Cline (.clinerules/*.md)
- Claude Code (CLAUDE.md)
- Goose (.goosehints)

Create rules once, deploy everywhere. Maintain consistency across all your AI tools.

EXAMPLES:
    rulesify rule new typescript-style    # Create a new rule
    rulesify deploy --all                 # Deploy all rules to all tools
    rulesify deploy --tool cursor --all   # Deploy to Cursor only
    rulesify config show                  # Show current configuration
    rulesify sync --dry-run               # Preview sync changes"#
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, global = true, help = "Path to custom config file")]
    pub config: Option<PathBuf>,

    #[arg(long, global = true, help = "Enable verbose output")]
    pub verbose: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage rules (create, edit, list, show, delete)
    Rule {
        #[command(subcommand)]
        action: commands::rule::RuleAction,
    },
    /// Deploy rules to AI tools (cursor, cline, claude-code, goose)
    Deploy {
        #[arg(long, help = "Target tool: cursor, cline, claude-code, or goose")]
        tool: Option<String>,
        #[arg(long, help = "Deploy specific rule by name")]
        rule: Option<String>,
        #[arg(long, help = "Deploy all rules")]
        all: bool,
    },
    /// Import rules from AI tool formats to URF
    Import {
        #[arg(long, help = "Source tool: cursor, cline, claude-code, or goose")]
        tool: String,
        #[arg(help = "Path to the tool-specific rule file")]
        file: PathBuf,
        #[arg(long, help = "Override rule ID (default: derived from filename)")]
        rule_id: Option<String>,
    },
    /// Validate rules for quality and format compliance
    Validate {
        #[arg(help = "Rule name to validate (use --all for all rules)")]
        rule: Option<String>,
        #[arg(long, help = "Validate all rules")]
        all: bool,
    },
    /// Synchronize deployed rules back to URF format
    Sync {
        #[arg(long, help = "Preview changes without applying them")]
        dry_run: bool,
        #[arg(long, help = "Sync specific rule only")]
        rule: Option<String>,
        #[arg(long, help = "Sync from specific tool only")]
        tool: Option<String>,
    },
    /// Manage configuration (show, edit, set storage location)
    Config {
        #[command(subcommand)]
        action: commands::config::ConfigAction,
    },
}

impl Cli {
    pub fn execute(self) -> anyhow::Result<()> {
        debug!("Executing CLI command: {:?}", self.command);

        let result = match self.command {
            Commands::Rule { action } => {
                debug!("Executing rule command: {:?}", action);
                commands::rule::run(action, self.config)
            }
            Commands::Deploy { tool, rule, all } => {
                debug!(
                    "Executing deploy command: tool={:?}, rule={:?}, all={}",
                    tool, rule, all
                );
                commands::deploy::run(tool, rule, all, self.config)
            }
            Commands::Import {
                tool,
                file,
                rule_id,
            } => {
                debug!(
                    "Executing import command: tool={}, file={:?}, rule_id={:?}",
                    tool, file, rule_id
                );
                commands::import::run(tool, file, rule_id, self.config)
            }
            Commands::Validate { rule, all } => {
                debug!("Executing validate command: rule={:?}, all={}", rule, all);
                commands::validate::run(rule, all, self.config)
            }
            Commands::Sync {
                dry_run,
                rule,
                tool,
            } => {
                debug!(
                    "Executing sync command: dry_run={}, rule={:?}, tool={:?}",
                    dry_run, rule, tool
                );
                commands::sync::run(dry_run, rule, tool, self.config)
            }
            Commands::Config { action } => {
                debug!("Executing config command: {:?}", action);
                commands::config::run(action, self.config)
            }
        };

        if let Err(ref e) = result {
            error!("Command execution failed: {}", e);
        }

        result
    }
}
