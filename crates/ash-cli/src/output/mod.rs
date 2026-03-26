//! Output formatting modules for Ash CLI
//!
//! Provides structured output formats for machine-readable diagnostic
//! and verification information.

pub mod json;

pub use json::{JsonDiagnostic, JsonLocation, JsonOutput, JsonTiming, JsonVerification};
