//! CLI command implementations for target Ash tooling.

pub mod check;
pub mod daemon;
pub mod dot;
pub mod fmt;
pub mod repl;
pub mod run;
pub mod template;
pub mod test;
pub mod trace;

pub use check::{CheckArgs, CheckOutputFormat, check};
pub use daemon::{DaemonArgs, daemon};
pub use dot::{DotArgs, dot};
pub use fmt::{FmtArgs, fmt};
pub use repl::{ReplArgs, repl};
pub use run::{RunArgs, RunOutputFormat, run};
pub use template::{TemplateArgs, template};
pub use test::{TestArgs, TestOutputFormat, test};
pub use trace::{TraceArgs, TraceExportFormat, trace};
