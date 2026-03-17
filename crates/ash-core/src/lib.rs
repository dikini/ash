//! Ash Core - IR and semantics definitions
//!
//! This crate defines the core abstract syntax, effects, and types
//! for the Ash workflow language.

pub mod ast;
pub mod effect;
pub mod provenance;
pub mod value;

// Property testing helpers available when proptest feature enabled
#[cfg(any(feature = "proptest-helpers", test))]
pub mod proptest_helpers;

pub use ast::*;
pub use effect::*;
pub use provenance::*;
pub use value::*;
