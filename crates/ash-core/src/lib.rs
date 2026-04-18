//! Ash Core - IR and semantics definitions
//!
//! This crate defines the core abstract syntax, effects, and types
//! for the Ash workflow language.

pub mod adt;
pub mod ast;
pub mod capabilities;
pub mod capability;
pub mod effect;
pub mod env_frame;
pub mod module_graph;
pub mod provenance;
pub mod small_step;
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

pub use env_frame::{BindingSlot, EnvFrame};

// Compile-time verification that key types are Send + Sync
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Value>();
    assert_send_sync::<Expr>();
    assert_send_sync::<EnvFrame>();
    assert_send_sync::<BindingSlot>();
};
