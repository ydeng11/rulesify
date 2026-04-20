pub mod init;
pub mod skill;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rulesify")]
#[command(about = "Discover and install AI agent skills")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Interactive setup to discover and install skills
    Init,

    /// Manage installed skills
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
}

#[derive(Subcommand)]
pub enum SkillCommands {
    /// List installed skills
    List,

    /// Add a skill from registry
    Add {
        /// Skill ID to add
        id: String,
        /// Install to global skill directory instead of project
        #[arg(long)]
        global: bool,
        /// Output instructions for AI agent instead of executing
        #[arg(long)]
        agent_mode: bool,
    },

    /// Remove an installed skill
    Remove {
        /// Skill ID to remove
        id: String,
        /// Remove from global skill directory instead of project
        #[arg(long)]
        global: bool,
        /// Output instructions for AI agent instead of executing
        #[arg(long)]
        agent_mode: bool,
    },

    /// Update registry cache
    Update {
        /// Output instructions for AI agent instead of executing
        #[arg(long)]
        agent_mode: bool,
    },
}

pub async fn run(cli: Cli) -> crate::utils::Result<()> {
    match cli.command {
        Commands::Init => init::run(cli.verbose).await?,
        Commands::Skill { command } => skill::run(command, cli.verbose).await?,
    }
    Ok(())
}
