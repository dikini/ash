//! Test command for Ash workflows.
//!
//! TASK-509: `ash test` CLI command.
//!
//! Usage: `ash test [PATH] [OPTIONS]`
//!
//! Default behavior:
//! - Discovers authored tests in conventional roots
//! - Runs them with per-test isolation
//! - Reports results in human or JSON format
//! - Does NOT run synthesized tests by default

use crate::error::{CliError, CliResult};
use crate::test_runner::executor::{self, SuiteConfig, SynthesizedSources};
use crate::test_runner::output;
use clap::Args;
use std::path::PathBuf;

/// Output format for test command
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum TestOutputFormat {
    /// Human-readable format
    #[default]
    Human,
    /// JSON format
    Json,
}

/// Synthesized test source selection.
/// Comma-separated list of: contracts, policies, obligations, laws
#[derive(Debug, Clone, Default)]
pub struct SynthesizedSourceList {
    pub contracts: bool,
    pub policies: bool,
    pub obligations: bool,
    pub laws: bool,
}

impl std::str::FromStr for SynthesizedSourceList {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut result = Self::default();
        for part in s.split(',') {
            match part.trim() {
                "contracts" => result.contracts = true,
                "policies" => result.policies = true,
                "obligations" => result.obligations = true,
                "laws" => result.laws = true,
                "" => {}
                other => return Err(format!("unknown synthesized source: {other}")),
            }
        }
        Ok(result)
    }
}

/// Arguments for the test command
#[derive(Args, Debug, Clone)]
pub struct TestArgs {
    /// Path to test file or directory (default: current directory)
    #[arg(value_name = "PATH", default_value = ".")]
    pub path: PathBuf,

    /// Output format (human, json)
    #[arg(short = 'f', long, value_enum, default_value = "human")]
    pub format: TestOutputFormat,

    /// Filter tests by tag
    #[arg(long)]
    pub tag: Option<String>,

    /// Filter tests by kind (unit, integration, e2e, property, smallworld)
    #[arg(long)]
    pub kind: Option<String>,

    /// Include synthesized tests from specified sources (contracts,policies,obligations,laws)
    #[arg(long, value_name = "SOURCES")]
    pub include_synthesized: Option<SynthesizedSourceList>,

    /// Only run synthesized tests from specified sources (implies --include-synthesized)
    #[arg(long, value_name = "SOURCES")]
    pub only_synthesized: Option<SynthesizedSourceList>,

    /// Stop on first failure
    #[arg(long)]
    pub fail_fast: bool,

    /// Default timeout per test in milliseconds
    #[arg(long, default_value = "30000")]
    pub timeout: u64,

    /// Seed for property tests (for reproducibility)
    #[arg(long)]
    pub seed: Option<u64>,

    /// Maximum number of cases for property tests
    #[arg(long)]
    pub max_cases: Option<usize>,

    /// Maximum number of worlds for small-world tests
    #[arg(long)]
    pub max_worlds: Option<usize>,
}

/// Run the test command.
pub fn test(args: &TestArgs) -> CliResult<()> {
    // Determine synthesized sources
    let synthesized_sources = if let Some(ref only) = args.only_synthesized {
        SynthesizedSources {
            contracts: only.contracts,
            policies: only.policies,
            obligations: only.obligations,
            laws: only.laws,
        }
    } else if let Some(ref include) = args.include_synthesized {
        SynthesizedSources {
            contracts: include.contracts,
            policies: include.policies,
            obligations: include.obligations,
            laws: include.laws,
        }
    } else {
        SynthesizedSources::default()
    };

    let include_synthesized = args.include_synthesized.is_some() || args.only_synthesized.is_some();
    let only_synthesized = args.only_synthesized.is_some();

    let config = SuiteConfig {
        root: args.path.clone(),
        format: args.format,
        tag_filter: args.tag.clone(),
        kind_filter: args.kind.clone(),
        include_synthesized,
        only_synthesized,
        synthesized_sources,
        synthesized_snapshots: Vec::new(),
        fail_fast: args.fail_fast,
        timeout_ms: args.timeout,
        seed: args.seed,
        max_cases: args.max_cases,
        max_worlds: args.max_worlds,
    };

    // Run the test suite (authored + synthesized based on config)
    let suite = executor::run_suite(&config);

    // Format and print output
    let output_str = match args.format {
        TestOutputFormat::Human => output::format_human(&suite),
        TestOutputFormat::Json => output::format_json(&suite)
            .map_err(|e| CliError::general(format!("JSON serialization error: {e}")))?,
    };

    print!("{output_str}");

    // Return error if any test failed
    if !suite.is_success() {
        return Err(CliError::general(format!(
            "{} of {} tests failed",
            suite.failed(),
            suite.total()
        )));
    }

    Ok(())
}
