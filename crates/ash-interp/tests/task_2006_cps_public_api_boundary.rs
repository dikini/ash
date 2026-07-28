//! TASK-2037: downstream checked-CPS public API boundary regression test.
//!
//! The `ash-interp` package may retain non-evaluator runtime support during
//! the Phase-205 migration, but an external crate must not reach checked CPS
//! validation or evaluation through it. In Rust, making `cps` non-public is
//! the actual library boundary: it hides `validate`, `eval_checked`,
//! `eval_unchecked`, and `eval_checked_terminal` together rather than relying
//! on convention at individual call sites.

const INTERP_LIBRARY_SOURCE: &str = include_str!("../src/lib.rs");

#[test]
fn external_consumers_cannot_reach_checked_cps_validation_or_evaluation() {
    assert!(
        !INTERP_LIBRARY_SOURCE.contains("pub mod cps;"),
        "a public `ash_interp::cps` module exposes the checked-CPS validation and evaluator \
         surface (`validate`, `eval_checked`, `eval_unchecked`, and \
         `eval_checked_terminal`) to non-Engine consumers"
    );
    assert!(
        !INTERP_LIBRARY_SOURCE.contains("pub use cps::"),
        "the residual runtime-support crate must not re-export any checked-CPS API"
    );
}
