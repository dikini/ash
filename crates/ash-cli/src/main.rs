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

use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

use ash_cli::commands::{CheckArgs, DotArgs, ReplArgs, RunArgs, TraceArgs};
use ash_cli::commands::{check, dot, repl, run, trace};
use ash_cli::error::{CliError, CliResult};

/// Color output options
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum ColorOption {
    /// Automatically detect color support
    #[default]
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

/// Ash CLI - A workflow language for governed AI systems
#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash - A workflow language for governed AI systems")]
#[command(version)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Suppress non-error output
    #[arg(long, short, global = true)]
    quiet: bool,

    /// Control color output
    #[arg(long, value_enum, default_value = "auto", global = true)]
    color: ColorOption,

    /// Increase verbosity (repeatable: -v, -vv, -vvv)
    #[arg(short, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
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
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Set up color output
    init_color(cli.color);

    // Set up logging/tracing based on verbosity
    init_logging(cli.verbose);

    // Execute the appropriate command
    let result = execute_command(&cli).await;

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            if !cli.quiet {
                eprintln!("{}: {}", "error".red().bold(), e);

                // Show help for specific error types
                if let CliError::UnknownCommand { .. } = &e {
                    eprintln!("\nRun 'ash --help' for available commands.");
                }
            }
            e.exit_code()
        }
    }
}

/// Execute the appropriate command based on CLI arguments
async fn execute_command(cli: &Cli) -> CliResult<()> {
    match &cli.command {
        Commands::Check(args) => {
            tracing::info!("Running check command for: {}", args.path);
            check::check(args).map_err(CliError::from)
        }
        Commands::Run(args) => {
            tracing::info!("Running workflow: {}", args.path);
            run::run(args).await.map_err(CliError::from)
        }
        Commands::Trace(args) => {
            tracing::info!("Tracing workflow: {}", args.path);
            trace::trace(args).await.map_err(CliError::from)
        }
        Commands::Repl(args) => {
            tracing::info!("Starting REPL");
            repl::repl(args).await.map_err(CliError::from)
        }
        Commands::Dot(args) => {
            tracing::info!("Generating DOT for: {}", args.path);
            dot::dot(args).map_err(CliError::from)
        }
    }
}

/// Initialize logging based on verbosity level
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => tracing::Level::INFO,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };

    tracing_subscriber::fmt().with_max_level(level).init();
}

/// Initialize color output based on color option
fn init_color(color: ColorOption) {
    match color {
        ColorOption::Never => {
            // Disable colors
            colored::control::set_override(false);
        }
        ColorOption::Always => {
            // Force enable colors
            colored::control::set_override(true);
        }
        ColorOption::Auto => {
            // Let colored detect automatically (default behavior)
            colored::control::unset_override();
        }
    }
}
