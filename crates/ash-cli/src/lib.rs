//! Ash CLI - Command-line interface for target Ash programs.
//!
//! This crate provides commands for checking, running, tracing, testing,
//! formatting, and inspecting target Ash source.
//!
//! # Commands
//!
//! - `check` - Type check Ash source files
//! - `run` - Execute target Ash entries
//! - `trace` - Run target Ash entries with provenance tracing
//! - `test` - Run tests (Phase 76 / TASK-509)
//! - `repl` - Interactive REPL for target Ash evaluation
//!
//! # Example
//!
//! ```bash
//! ash check main.ash
//! ash run main.ash
//! ash test tests/ash/
//! ```

pub mod commands;
pub mod error;
pub mod output;
pub mod templates;
pub mod test_runner;
pub mod value_convert;

pub use commands::*;
pub use error::{CliError, CliResult};
pub use output::*;
pub use test_runner::types::{Outcome, TestKind, TestResult, TestSource, TestSuiteResult};
pub use value_convert::{json_to_value, value_to_json};
