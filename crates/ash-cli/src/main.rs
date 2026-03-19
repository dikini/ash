//! Ash CLI - Command-line interface for the Ash workflow language
//!
//! # Phase 6: CLI Implementation
//!
//! This module implements the CLI with the following commands:
//! - `check` - Type check workflow files (TASK-053)
//! - `run` - Execute workflows (TASK-054)
//! - `trace` - Run workflows with provenance tracing (TASK-055)
//! - `repl` - Interactive REPL (TASK-056)
//! - `dot` - Generate Graphviz DOT output (TASK-057)

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod commands;

use commands::{CheckArgs, DotArgs, ReplArgs, RunArgs, TraceArgs, check, dot, repl, run, trace};

/// Ash CLI - A workflow language for governed AI systems
#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash - A workflow language for governed AI systems")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

/// CLI subcommands
#[derive(Subcommand)]
enum Commands {
    /// Type check workflow files (TASK-053)
    #[command(name = "check", about = "Type check workflow files")]
    Check(CheckArgs),

    /// Execute a workflow (TASK-054)
    #[command(name = "run", about = "Execute a workflow")]
    Run(RunArgs),

    /// Run workflow with provenance tracing (TASK-055)
    #[command(name = "trace", about = "Run workflow with provenance tracing")]
    Trace(TraceArgs),

    /// Start interactive REPL (TASK-056)
    #[command(name = "repl", about = "Start interactive REPL")]
    Repl(ReplArgs),

    /// Generate Graphviz DOT output (TASK-057)
    #[command(name = "dot", about = "Generate Graphviz DOT output")]
    Dot(DotArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Set up logging/tracing
    if cli.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // Handle color setting
    if cli.no_color {
        unsafe {
            std::env::set_var("NO_COLOR", "1");
        }
    }

    // Execute the appropriate command
    match cli.command {
        Commands::Check(args) => {
            tracing::info!("Running check command for: {}", args.path);
            if let Err(e) = check::check(&args) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Run(args) => {
            tracing::info!("Running workflow: {}", args.path);
            if let Err(e) = run::run(&args).await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Trace(args) => {
            tracing::info!("Tracing workflow: {}", args.path);
            if let Err(e) = trace::trace(&args).await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Repl(args) => {
            tracing::info!("Starting REPL");
            if let Err(e) = repl::repl(&args).await {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Dot(args) => {
            tracing::info!("Generating DOT for: {}", args.path);
            if let Err(e) = dot::dot(&args) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
