use anyhow::Result;
use clap::{Command, CommandFactory};
use clap_complete::{generate, Generator, Shell};
use std::io;
use std::path::PathBuf;
use crate::cli::Cli;

pub fn run(shell: Shell, _config_path: Option<PathBuf>) -> Result<()> {
    let mut cmd = Cli::command();
    print_completions(shell, &mut cmd);
    Ok(())
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
