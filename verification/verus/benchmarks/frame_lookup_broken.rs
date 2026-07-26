//! Deliberately broken TASK-1993 lemma candidate.
//!
//! This file is expected to be rejected.  It represents the pre-repair
//! candidate in the hybrid proof-repair benchmark; it makes the false claim
//! that a nonmatching inner frame shadows a prior matching frame.

use vstd::prelude::*;

fn main() {}

verus! {
pub struct Frame { pub op: int }
pub open spec fn lookup(stack: Seq<Frame>, target: int) -> Option<int>
    decreases stack.len(),
{
    if stack.len() == 0 { None }
    else if stack.last().op == target { Some(stack.len() - 1) }
    else { lookup(stack.drop_last(), target) }
}

pub proof fn broken_nonmatch_shadows(stack: Seq<Frame>, target: int, frame: Frame)
    requires frame.op != target,
    ensures lookup(stack.push(frame), target) == Some(stack.len() as int),
{
    // Expected verifier failure: a nonmatching inner frame preserves lookup.
}
} // verus!
