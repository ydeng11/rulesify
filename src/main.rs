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