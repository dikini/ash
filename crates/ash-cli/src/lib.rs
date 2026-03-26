//! Ash CLI - Command-line interface for the Ash workflow language.
//!
//! This crate provides the CLI for the Ash workflow language, including
//! commands for checking, running, tracing, and visualizing workflows.
//!
//! # Commands
//!
//! - `check` - Type check workflow files
//! - `run` - Execute workflows
//! - `trace` - Run workflows with provenance tracing
//! - `repl` - Interactive REPL for workflow evaluation
//! - `dot` - Generate Graphviz DOT output
//!
//! # Example
//!
//! ```bash
//! ash check workflow.ash
//! ash run workflow.ash --input '{"x": 42}'
//! ash dot workflow.ash --output graph.dot
//! ```

pub mod commands;
pub mod error;
pub mod output;
pub mod value_convert;

pub use commands::*;
pub use error::{CliError, CliResult};
pub use output::*;
pub use value_convert::{json_to_value, value_to_json};
