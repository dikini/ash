//! Minimal negative fixture for the isolated TASK-1991 Verus runner.
//!
//! The verifier must reject this unsound postcondition.  A successful result
//! is therefore evidence of a broken runner contract, not a passing proof.

use vstd::prelude::*;

fn main() {}

verus! {
    proof fn one_is_two()
        ensures 1int == 2int,
    {
    }
}
