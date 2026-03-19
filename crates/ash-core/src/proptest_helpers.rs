//! Property testing helpers for Ash Core
//!
//! This module provides `proptest` strategies for generating valid
//! Ash values, effects, and patterns for testing.
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

use crate::Name;
use crate::ast::{Expr, Pattern};
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
                prop::collection::vec(inner.clone(), 0..10).prop_map(|v| Value::List(Box::new(v))),
                prop::collection::hash_map("[a-z]+".prop_map(String::from), inner, 0..10)
                    .prop_map(|m| Value::Record(Box::new(m))),
            ]
        },
    )
}

/// Generate arbitrary identifier names
pub fn arb_name() -> impl proptest::strategy::Strategy<Value = Name> {
    use proptest::prelude::*;
    "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(String::from)
}

/// Generate arbitrary Pattern values
pub fn arb_pattern() -> impl proptest::strategy::Strategy<Value = Pattern> {
    use proptest::prelude::*;

    let leaf = prop_oneof![
        arb_name().prop_map(Pattern::Variable),
        Just(Pattern::Wildcard),
        arb_value().prop_map(Pattern::Literal),
    ];

    leaf.prop_recursive(
        4,  // Depth
        64, // Max size
        8,  // Items per collection
        |inner| {
            prop_oneof![
                // Tuple pattern
                prop::collection::vec(inner.clone(), 0..4).prop_map(Pattern::Tuple),
                // Record pattern
                prop::collection::vec((arb_name(), inner.clone()), 0..4).prop_map(Pattern::Record),
                // List pattern with optional rest
                (
                    prop::collection::vec(inner, 0..4),
                    proptest::option::of(arb_name())
                )
                    .prop_map(|(prefix, rest)| Pattern::List(prefix, rest)),
            ]
        },
    )
}

/// Generate simple expressions (for use in workflows)
pub fn arb_expr() -> impl proptest::strategy::Strategy<Value = Expr> {
    use proptest::prelude::*;
    prop_oneof![
        arb_value().prop_map(Expr::Literal),
        arb_name().prop_map(Expr::Variable),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_arb_effect_generates_all_variants(e in arb_effect()) {
            // Just verify we can generate effects
            let _ = format!("{:?}", e);
        }

        #[test]
        fn test_arb_value_roundtrips(v in arb_value()) {
            let json = serde_json::to_string(&v).expect("serialize");
            let v2: Value = serde_json::from_str(&json).expect("deserialize");
            prop_assert_eq!(v, v2);
        }

        #[test]
        fn test_arb_pattern_bindings_unique(pat in arb_pattern()) {
            let bindings = pat.bindings();
            let unique: std::collections::HashSet<_> = bindings.iter().collect();
            prop_assert_eq!(bindings.len(), unique.len(), "Bindings should be unique");
        }

        #[test]
        fn test_arb_name_is_valid_identifier(name in arb_name()) {
            // Name should start with letter or underscore
            prop_assert!(
                name.chars().next().map(|c| c.is_ascii_alphabetic() || c == '_').unwrap_or(false),
                "Name should start with letter or underscore"
            );
            // Rest should be alphanumeric or underscore
            prop_assert!(
                name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'),
                "Name should be alphanumeric or underscore"
            );
        }
    }
}
