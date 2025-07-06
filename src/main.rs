use clap::Parser;
use rulesify::cli::Cli;

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    if let Err(e) = cli.execute() {
        eprintln!("Error: {}", e);

        // Log the error chain for debugging
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }

        std::process::exit(1);
    }
}
