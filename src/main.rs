use clap::Parser;
use rulesify::cli::Cli;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
