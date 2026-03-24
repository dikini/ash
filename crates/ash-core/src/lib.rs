//! Ash Core - IR and semantics definitions
//!
//! This crate defines the core abstract syntax, effects, and types
//! for the Ash workflow language.

pub mod ast;
pub mod effect;
pub mod module_graph;
pub mod provenance;
pub mod stream;
pub mod value;
pub mod visualize;
pub mod workflow_contract;

// Property testing helpers available when proptest feature enabled
#[cfg(any(feature = "proptest-helpers", test))]
pub mod proptest_helpers;

// Testing helpers available in test mode
#[cfg(test)]
pub mod test_helpers;

pub use ast::*;
pub use effect::*;
pub use provenance::*;
pub use stream::{
    Mailbox, MailboxEntry, MailboxOverflowError, OverflowStrategy, Receive as StreamReceive,
    ReceiveArm as StreamReceiveArm, ReceiveMode as StreamReceiveMode, StreamRef,
};
pub use value::*;
pub use visualize::*;
