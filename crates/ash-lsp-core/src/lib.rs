//! Ash LSP Core
//!
//! Core types and logic for the Ash Language Server Protocol implementation.
//!
//! This crate provides:
//! - **vfs** — Virtual File System for tracking open documents
//! - **diagnostics** — Diagnostic aggregation (parse → typeck → lint pipeline)
//! - **analysis** — Analysis cache with change detection

// Pedantic warnings that are too noisy for this crate.
#![allow(
    clippy::option_if_let_else,
    clippy::manual_let_else,
    clippy::significant_drop_tightening,
    clippy::cast_possible_truncation,
    missing_docs
)]

pub mod analysis;
pub mod diagnostics;
pub mod vfs;
