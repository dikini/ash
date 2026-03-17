//! Property testing helpers for Ash Core
//!
//! This module provides `proptest` strategies for generating valid
//! Ash values and effects for testing.
//!
//! # Example
//! ```ignore
//! use ash_core::proptest_helpers::*;
//! use proptest::prelude::*;
//!
//! proptest! {
//!     #[test]
//!     fn test_effect_lattice_associativity(
//!         e1 in arb_effect(),
//!         e2 in arb_effect(),
//!         e3 in arb_effect()
//!     ) {
//!         assert_eq!(
//!             e1.join(e2).join(e3),
//!             e1.join(e2.join(e3))
//!         );
//!     }
//! }
//! ```

use crate::effect::Effect;
use crate::value::Value;

/// Generate arbitrary Effect values
pub fn arb_effect() -> impl proptest::strategy::Strategy<Value = Effect> {
    use proptest::prelude::*;
    prop_oneof![
        Just(Effect::Epistemic),
        Just(Effect::Deliberative),
        Just(Effect::Evaluative),
        Just(Effect::Operational),
    ]
}

/// Generate arbitrary Value values
pub fn arb_value() -> impl proptest::strategy::Strategy<Value = Value> {
    use proptest::prelude::*;

    let leaf = prop_oneof![
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(Value::Int),
        "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(Value::String),
    ];

    leaf.prop_recursive(
        8,   // Depth
        256, // Max size
        10,  // Items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..10).prop_map(Value::List),
                prop::collection::hash_map("[a-z]+".prop_map(String::from), inner, 0..10)
                    .prop_map(Value::Record),
            ]
        },
    )
}
