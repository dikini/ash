//! Ash CLI - Command-line interface for target Ash programs.
//!
//! # Phase 6: CLI Implementation
//!
//! This module implements the CLI with the following commands:
//! - `check` - Type check Ash source files (TASK-053)
//! - `run` - Execute target Ash entries (TASK-054)
//! - `trace` - Run target Ash entries with provenance tracing (TASK-055)
//! - `repl` - Interactive REPL (TASK-056)
//! - `test` - Run tests (Phase 76 / TASK-509)
//! - `fmt` - Format Ash source files (Phase 200)

use std::process::ExitCode;

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;

use ash_cli::commands::{
    CheckArgs, DaemonArgs, FmtArgs, ReplArgs, RunArgs, TemplateArgs, TestArgs, TraceArgs,
};
use ash_cli::commands::{check, daemon, fmt, repl, run, template, test, trace};
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

/// Ash CLI - target Ash tooling for governed programs.
#[derive(Parser)]
#[command(name = "ash")]
#[command(about = "Ash - target language tooling for governed programs")]
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
    /// Type check Ash source files (TASK-053)
    #[command(name = "check", about = "Type check Ash source files")]
    Check(CheckArgs),

    /// Execute a target Ash entry (TASK-054)
    #[command(name = "run", about = "Execute a target Ash entry")]
    Run(RunArgs),

    /// Run a target Ash entry with provenance tracing (TASK-055)
    #[command(
        name = "trace",
        about = "Run a target Ash entry with provenance tracing"
    )]
    Trace(TraceArgs),

    /// Run tests (Phase 76 / TASK-509)
    #[command(name = "test", about = "Run Ash tests")]
    Test(TestArgs),

    /// Start interactive REPL (TASK-056)
    #[command(name = "repl", about = "Start interactive REPL")]
    Repl(ReplArgs),

    /// Format Ash source files (Phase 200)
    #[command(name = "fmt", about = "Format Ash source files")]
    Fmt(FmtArgs),

    /// Control the local RuntimeKernel daemon (TASK-929)
    #[command(name = "daemon", about = "Control the local RuntimeKernel daemon")]
    Daemon(DaemonArgs),

    /// Work with Ash app templates (Phase 199)
    #[command(name = "template", about = "Work with Ash app templates")]
    Template(TemplateArgs),
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
        Ok(code) => code,
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
async fn execute_command(cli: &Cli) -> CliResult<ExitCode> {
    match &cli.command {
        Commands::Check(args) => {
            tracing::info!("Running check command for: {}", args.path);
            check::check(args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Run(args) => {
            tracing::info!("Running Ash entry: {}", args.path);
            run::run(args)
                .await
                .map(|outcome| outcome.exit_code())
                .map_err(run::classify_run_cli_error)
        }
        Commands::Trace(args) => {
            tracing::info!("Tracing Ash entry: {}", args.path);
            trace::trace(args).await.map_err(CliError::from)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Test(args) => {
            tracing::info!("Running tests: {}", args.path.display());
            test::test(args)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Repl(args) => {
            tracing::info!("Starting REPL");
            repl::repl(args).await.map_err(CliError::from)?;
            Ok(ExitCode::SUCCESS)
        }
        Commands::Fmt(args) => {
            tracing::info!("Formatting Ash source");
            fmt::fmt(args)
        }
        Commands::Daemon(args) => {
            tracing::info!("Running daemon control command");
            daemon::daemon(args).await.map_err(CliError::from)
        }
        Commands::Template(args) => {
            tracing::info!("Running template command");
            template::template(args)?;
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Initialize logging based on verbosity level
///
/// Default is WARN to keep output clean. Use -v for INFO, -vv for DEBUG, -vvv for TRACE.
fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => tracing::Level::WARN,
        1 => tracing::Level::INFO,
        2 => tracing::Level::DEBUG,
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
