//! Ash test runner infrastructure.
//!
//! TASK-509: Runner substrate with result model, discovery, and output.
//! TASK-510: Per-test isolation and panic capture.
//! TASK-513: Synthesized test generation (opt-in).
//! TASK-514: Property and small-world execution (bounded, seeded).

pub mod discovery;
pub mod executor;
pub mod metadata;
pub mod output;
pub mod property;
pub mod synthesized;
pub mod types;

// Phase 144: Algebra law profiles for generated property tests
pub mod algebra_law_profile;

pub use types::{Outcome, TestKind, TestResult, TestSource, TestSuiteResult};
