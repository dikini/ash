//! Ash CLI - Command-line interface for the Ash workflow language

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash - A workflow language for governed AI systems")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a workflow
    Run {
        /// Path to workflow file
        path: String,
        /// Enable trace output
        #[arg(long)]
        trace: bool,
        /// Input parameters (JSON)
        #[arg(short, long)]
        input: Option<String>,
    },
    /// Type check a workflow
    Check {
        /// Path to workflow file
        path: String,
    },
    /// Parse and display AST
    Parse {
        /// Path to workflow file
        path: String,
    },
    /// Start interactive REPL
    Repl,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Run { path, trace, input } => {
            info!("Running workflow: {}", path);
            if trace {
                info!("Trace enabled");
            }
            if let Some(inp) = input {
                info!("Input: {}", inp);
            }
            // TODO: Implement actual execution
            warn!("Execution not yet implemented");
            Ok(())
        }
        Commands::Check { path } => {
            info!("Type checking: {}", path);
            // TODO: Implement type checking
            warn!("Type checking not yet implemented");
            Ok(())
        }
        Commands::Parse { path } => {
            info!("Parsing: {}", path);
            // TODO: Implement parsing
            warn!("Parsing not yet implemented");
            Ok(())
        }
        Commands::Repl => {
            info!("Starting REPL");
            println!("Ash REPL - Work in progress");
            Ok(())
        }
    }
}
