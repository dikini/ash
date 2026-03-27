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
use std::cell::Cell;
use std::rc::Rc;

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

/// Internal state for tracking unique binding names during pattern generation.
#[derive(Clone, Debug)]
struct PatternGenContext {
    counter: Rc<Cell<u64>>,
}

impl PatternGenContext {
    fn new() -> Self {
        Self {
            counter: Rc::new(Cell::new(0)),
        }
    }

    fn next_name(&self) -> Name {
        let id = self.counter.get();
        self.counter.set(id + 1);
        format!("G_{id}")
    }
}

/// Generate an arbitrary field name for record patterns (not a binding).
/// These don't need to be unique since they're field labels, not variable bindings.
fn arb_field_name() -> impl proptest::strategy::Strategy<Value = Name> {
    use proptest::prelude::*;
    "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(String::from)
}

/// Generate arbitrary Pattern values with unique binding names.
///
/// This strategy ensures that all variable bindings within a generated pattern
/// are unique, preventing conflicts between variables and rest patterns.
pub fn arb_pattern() -> impl proptest::strategy::Strategy<Value = Pattern> {
    use proptest::prelude::*;

    // Use a constant strategy to provide the context, then chain to the actual pattern generation
    Just(PatternGenContext::new()).prop_flat_map(|ctx| {
        arb_pattern_with_context(ctx, 4) // 4 is the max depth
    })
}

/// Internal function to generate patterns with unique binding tracking.
fn arb_pattern_with_context(
    ctx: PatternGenContext,
    max_depth: u32,
) -> impl proptest::strategy::Strategy<Value = Pattern> {
    use proptest::prelude::*;

    if max_depth == 0 {
        // At max depth, only generate leaf patterns
        prop_oneof![
            Just(ctx.clone()).prop_map(|c| Pattern::Variable(c.next_name())),
            Just(Pattern::Wildcard),
            arb_value().prop_map(Pattern::Literal),
        ]
        .boxed()
    } else {
        let leaf = prop_oneof![
            Just(ctx.clone()).prop_map(|c| Pattern::Variable(c.next_name())),
            Just(Pattern::Wildcard),
            arb_value().prop_map(Pattern::Literal),
        ];

        leaf.prop_recursive(
            max_depth,
            64, // Max size
            8,  // Items per collection
            move |inner| {
                let ctx_clone = ctx.clone();
                prop_oneof![
                    // Tuple pattern
                    prop::collection::vec(inner.clone(), 0..4).prop_map(Pattern::Tuple),
                    // Record pattern - field names don't need to be unique, but nested patterns do
                    prop::collection::vec((arb_field_name(), inner.clone()), 0..4)
                        .prop_map(Pattern::Record),
                    // List pattern with optional rest - rest name must be unique
                    (
                        prop::collection::vec(inner, 0..4),
                        proptest::option::of(Just(ctx_clone.clone()).prop_map(|c| c.next_name()))
                    )
                        .prop_map(|(prefix, rest)| Pattern::List(prefix, rest)),
                ]
            },
        )
        .boxed()
    }
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
