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

use commands::{check, dot, repl, run, trace, CheckArgs, DotArgs, ReplArgs, RunArgs, TraceArgs};

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

#[cfg(test)]
mod tests {
    
    use assert_cmd::Command;
    use predicates::prelude::*;
    
    

    #[test]
    fn test_cli_help() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.arg("--help");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Ash"))
            .stdout(predicate::str::contains("check"))
            .stdout(predicate::str::contains("run"))
            .stdout(predicate::str::contains("trace"))
            .stdout(predicate::str::contains("repl"))
            .stdout(predicate::str::contains("dot"));
    }

    #[test]
    fn test_check_help() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.args(["check", "--help"]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Type check"));
    }

    #[test]
    fn test_run_help() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.args(["run", "--help"]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Execute"));
    }

    #[test]
    fn test_dot_help() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.args(["dot", "--help"]);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("Graphviz"));
    }

    #[test]
    fn test_version() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.arg("--version");
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("0.1.0"));
    }

    #[test]
    fn test_check_nonexistent_file() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.args(["check", "nonexistent.ash"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }

    #[test]
    fn test_dot_nonexistent_file() {
        let mut cmd = Command::cargo_bin("ash").unwrap();
        cmd.args(["dot", "nonexistent.ash"]);
        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error:"));
    }
}
