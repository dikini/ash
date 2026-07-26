//! Minimal positive fixture for the isolated TASK-1991 Verus runner.
//!
//! This file is deliberately outside the Cargo workspace.  It is consumed
//! only by the pinned Verus runner recorded in the spike manifest.

use vstd::prelude::*;

fn main() {}

verus! {
    proof fn one_is_one()
        ensures 1int == 1int,
    {
    }
}
