use clap::{Parser, Subcommand};
use std::path::PathBuf;

pub mod commands;

#[derive(Parser)]
#[command(name = "rulesify")]
#[command(about = "A CLI tool for managing AI assistant rules")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
    
    #[arg(long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new rules project
    Init {
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        template: Option<String>,
    },
    /// Manage rules
    Rule {
        #[command(subcommand)]
        action: commands::rule::RuleAction,
    },
    /// Deploy rules to AI tools
    Deploy {
        #[arg(long)]
        tool: Option<String>,
        #[arg(long)]
        rule: Option<String>,
        #[arg(long)]
        all: bool,
    },
    /// Synchronize rules across all tools
    Sync {
        #[arg(long)]
        dry_run: bool,
    },
    /// Manage templates
    Template {
        #[command(subcommand)]
        action: commands::template::TemplateAction,
    },
}

impl Cli {
    pub fn execute(self) -> anyhow::Result<()> {
        match self.command {
            Commands::Init { name, template } => {
                commands::init::run(name, template)
            }
            Commands::Rule { action } => {
                commands::rule::run(action)
            }
            Commands::Deploy { tool, rule, all } => {
                commands::deploy::run(tool, rule, all)
            }
            Commands::Sync { dry_run } => {
                commands::sync::run(dry_run)
            }
            Commands::Template { action } => {
                commands::template::run(action)
            }
        }
    }
} 