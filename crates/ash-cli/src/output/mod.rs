//! Output formatting modules for Ash CLI
//!
//! Provides structured output formats for machine-readable diagnostic
//! and verification information.
//!
//! TASK-298: Updated to SPEC-005 compliant schema with diagnostics array and summary

pub mod json;

pub use json::{Diagnostic, JsonLocation, JsonOutput, JsonTiming, Severity, Summary};
