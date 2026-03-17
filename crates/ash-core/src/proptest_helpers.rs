//! Property testing helpers for Ash Core
//!
//! This module provides `proptest` strategies for generating valid
//! Ash programs, values, and configurations for testing.
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
use crate::workflow::Workflow;

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
        any::<f64>().prop_map(Value::Float),
        "[a-zA-Z_][a-zA-Z0-9_]*".prop_map(Value::Symbol),
        any::<Vec<u8>>().prop_map(Value::Bytes),
    ];
    
    leaf.prop_recursive(
        8,   // Depth
        256, // Max size
        10,  // Items per collection
        |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..10).prop_map(Value::List),
                prop::collection::hash_map("[a-z]+".prop_map(String::from), inner, 0..10)
                    .prop_map(Value::Map),
            ]
        },
    )
}

/// Generate Workflow nodes with specific effect levels
pub fn arb_effectful_workflow(effect: Effect) -> impl proptest::strategy::Strategy<Value = Workflow> {
    use proptest::prelude::*;
    
    match effect {
        Effect::Epistemic => arb_epistemic_workflow(),
        Effect::Deliberative => arb_deliberative_workflow(),
        Effect::Evaluative => arb_evaluative_workflow(),
        Effect::Operational => arb_operational_workflow(),
    }
}

fn arb_epistemic_workflow() -> impl proptest::strategy::Strategy<Value = Workflow> {
    use proptest::prelude::*;
    // OBSERVE workflows
    ("[a-z]+".prop_map(String::from), "[a-z]+".prop_map(String::from))
        .prop_map(|(cap, bind)| Workflow::Observe { capability: cap, binding: bind })
}

fn arb_deliberative_workflow() -> impl proptest::strategy::Strategy<Value = Workflow> {
    use proptest::strategy::Strategy;
    // TODO: Implement when Workflow types are complete
    arb_epistemic_workflow()
}

fn arb_evaluative_workflow() -> impl proptest::strategy::Strategy<Value = Workflow> {
    use proptest::strategy::Strategy;
    // TODO: Implement when Workflow types are complete
    arb_epistemic_workflow()
}

fn arb_operational_workflow() -> impl proptest::strategy::Strategy<Value = Workflow> {
    use proptest::strategy::Strategy;
    // TODO: Implement when Workflow types are complete
    arb_epistemic_workflow()
}

/// Generate guaranteed-conflicting policy sets for SMT testing
#[cfg(feature = "smt")]
pub fn arb_conflicting_policies() -> impl proptest::strategy::Strategy<Value = Vec<crate::policy::Policy>> {
    use proptest::prelude::*;
    use crate::policy::Policy;
    
    prop_oneof![
        // Budget conflicts: min > max
        (100u64..1000, 1u64..100).prop_map(|(min, gap)| vec![
            Policy::MinBudget { min },
            Policy::Budget { max: min - gap },
        ]),
        // Time range conflicts: disjoint ranges
        (0u8..12, 12u8..24).prop_map(|(end, start)| vec![
            Policy::TimeRange { start_hour: 0, end_hour: end },
            Policy::TimeRange { start_hour: start, end_hour: 24 },
        ]),
        // Region conflicts
        Just(vec![
            Policy::Region { allowed: vec!["us-east-1".into()] },
            Policy::Region { allowed: vec!["eu-west-1".into()] },
        ]),
    ]
}

/// Generate valid (non-conflicting) policy sets
#[cfg(feature = "smt")]
pub fn arb_valid_policies() -> impl proptest::strategy::Strategy<Value = Vec<crate::policy::Policy>> {
    use proptest::prelude::*;
    use crate::policy::Policy;
    
    // Policies that are definitely satisfiable together
    (100u64..1000, 2000u64..5000).prop_map(|(min, max)| vec![
        Policy::MinBudget { min },
        Policy::Budget { max },
    ])
}

/// Generate workflow expressions with specific effect levels
pub fn arb_effectful_expr(effect: Effect) -> impl proptest::strategy::Strategy<Value = crate::expr::Expr> {
    use proptest::prelude::*;
    use crate::expr::Expr;
    
    match effect {
        Effect::Epistemic => {
            // Pure expressions (variables, literals)
            arb_value().prop_map(Expr::Const).boxed()
        }
        Effect::Deliberative => {
            // Analysis expressions
            (arb_effectful_expr(Effect::Epistemic), "[a-z]+".prop_map(String::from))
                .prop_map(|(e, op)| Expr::Analyze { expr: Box::new(e), operation: op })
                .boxed()
        }
        Effect::Evaluative => {
            // Guard expressions
            arb_effectful_expr(Effect::Deliberative)
                .prop_map(|e| Expr::Guard(Box::new(e)))
                .boxed()
        }
        Effect::Operational => {
            // Action expressions
            ("[a-z]+".prop_map(String::from), arb_value())
                .prop_map(|(cap, arg)| Expr::Action { capability: cap, argument: arg })
                .boxed()
        }
    }
}
