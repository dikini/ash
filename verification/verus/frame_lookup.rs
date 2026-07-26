//! TASK-1993 frame-ordered operation-dispatch pilot.
//!
//! This is an isolated Verus model of the *selection* algorithm used by
//! `HandlerChain::find_operation_frame`: scan a finite frame stack from its
//! innermost end and select the first frame whose operation equals the query.
//! It is deliberately not a proof about production Rust.  In particular,
//! there is no verified adapter from `HandlerFrame`, `EffectOp`, `Name`, or
//! `HandlerClause` into these integer atoms.  Evaluating a provider, handler
//! body, or resume continuation is outside this model.

use vstd::prelude::*;

fn main() {}

verus! {

pub type Operation = int;
pub type Payload = int;

/// `kind` distinguishes shallow-handler (`false`) from provider (`true`).
/// Selection deliberately ignores it: both frame kinds share one ordering.
pub struct ModelFrame {
    pub kind: bool,
    pub op: Operation,
    pub payload: Payload,
}

pub type FrameStack = Seq<ModelFrame>;

pub open spec fn frame_matches(frame: ModelFrame, target: Operation) -> bool {
    frame.op == target
}

/// The canonical innermost-first lookup model.  The result is an index into
/// the original stack, so callers retain frame-kind and payload provenance.
pub open spec fn lookup_frame(stack: FrameStack, target: Operation) -> Option<int>
    decreases stack.len(),
{
    if stack.len() == 0 {
        None
    } else if frame_matches(stack.last(), target) {
        Some(stack.len() - 1)
    } else {
        lookup_frame(stack.drop_last(), target)
    }
}

pub open spec fn no_matching_frame(stack: FrameStack, target: Operation) -> bool {
    forall |i: int| 0 <= i < stack.len() ==> !frame_matches(stack[i], target)
}

/// The payload projection of a selected frame.  Keeping the lookup index in
/// the result model prevents handler/provider payloads from being fabricated
/// independently of their source frame.
pub open spec fn selected_payload(stack: FrameStack, target: Operation) -> Option<Payload> {
    match lookup_frame(stack, target) {
        Some(index) => Some(stack[index].payload),
        None => None,
    }
}

/// An absent lookup means that no frame in the finite stack matches.
pub proof fn lookup_absent_iff_no_match(stack: FrameStack, target: Operation)
    ensures lookup_frame(stack, target) == None <==> no_matching_frame(stack, target),
    decreases stack.len(),
{
    if stack.len() > 0 {
        let prefix = stack.drop_last();
        let last = stack.last();
        stack.lemma_add_last_back();
        lookup_absent_iff_no_match(prefix, target);
        if frame_matches(last, target) {
            assert(lookup_frame(stack, target) != None);
            assert(!no_matching_frame(stack, target));
        } else {
            assert(lookup_frame(stack, target) == lookup_frame(prefix, target));
            assert(no_matching_frame(stack, target) == no_matching_frame(prefix, target));
        }
    }
}

/// A selected index is in bounds and denotes a matching source frame.
pub proof fn lookup_result_is_bounded_matching(
    stack: FrameStack,
    target: Operation,
    index: int,
)
    requires lookup_frame(stack, target) == Some(index),
    ensures 0 <= index < stack.len(), frame_matches(stack[index], target),
    decreases stack.len(),
{
    let last_index = stack.len() - 1;
    if frame_matches(stack.last(), target) {
        assert(index == last_index);
    } else {
        assert(lookup_frame(stack.drop_last(), target) == Some(index));
        lookup_result_is_bounded_matching(stack.drop_last(), target, index);
        assert(stack.drop_last().len() == stack.len() - 1);
        assert(index < stack.drop_last().len());
        assert(stack[index] == stack.drop_last()[index]);
    }
}

/// The selected index is the greatest matching (innermost) index.
pub proof fn lookup_result_is_greatest_matching(
    stack: FrameStack,
    target: Operation,
    index: int,
)
    requires lookup_frame(stack, target) == Some(index),
    ensures forall |other: int|
        0 <= other < stack.len() && frame_matches(stack[other], target) ==> other <= index,
    decreases stack.len(),
{
    lookup_result_is_bounded_matching(stack, target, index);
    let last_index = stack.len() - 1;
    if frame_matches(stack.last(), target) {
        assert(index == last_index);
    } else {
        assert(lookup_frame(stack.drop_last(), target) == Some(index));
        lookup_result_is_greatest_matching(stack.drop_last(), target, index);
        assert forall |other: int|
            0 <= other < stack.len() && frame_matches(stack[other], target) implies other <= index by {
                if other < stack.drop_last().len() {
                    assert(stack[other] == stack.drop_last()[other]);
                } else {
                    assert(other == last_index);
                    assert(!frame_matches(stack[other], target));
                }
            }
    }
}

/// The result index preserves the exact payload stored in its source frame.
pub proof fn lookup_payload_provenance(
    stack: FrameStack,
    target: Operation,
    index: int,
)
    requires lookup_frame(stack, target) == Some(index),
    ensures selected_payload(stack, target) == Some(stack[index].payload),
{
    lookup_result_is_bounded_matching(stack, target, index);
}

/// Appending a matching handler or provider shadows every earlier match.
pub proof fn append_matching_frame_shadows(
    stack: FrameStack,
    target: Operation,
    frame: ModelFrame,
)
    requires frame_matches(frame, target),
    ensures lookup_frame(stack.push(frame), target) == Some(stack.len() as int),
{
    assert(stack.push(frame).last() == frame);
    assert(stack.push(frame).drop_last() == stack);
}

/// An innermost nonmatching frame does not perturb lookup of an earlier match.
pub proof fn append_nonmatching_frame_preserves(
    stack: FrameStack,
    target: Operation,
    frame: ModelFrame,
)
    requires !frame_matches(frame, target),
    ensures lookup_frame(stack.push(frame), target) == lookup_frame(stack, target),
{
    assert(stack.push(frame).last() == frame);
    assert(stack.push(frame).drop_last() == stack);
}

/// Handler/provider kind is not an ordering discriminator.  Equal operation
/// projections therefore produce the same selected index even when every
/// payload and frame kind differs.
pub proof fn lookup_is_kind_agnostic(
    left: FrameStack,
    right: FrameStack,
    target: Operation,
)
    requires
        left.len() == right.len(),
        forall |i: int| 0 <= i < left.len() ==> left[i].op == right[i].op,
    ensures lookup_frame(left, target) == lookup_frame(right, target),
    decreases left.len(),
{
    if left.len() > 0 {
        let left_prefix = left.drop_last();
        let right_prefix = right.drop_last();
        left.lemma_add_last_back();
        right.lemma_add_last_back();
        assert(left_prefix.len() == right_prefix.len());
        assert forall |i: int| 0 <= i < left_prefix.len() implies left_prefix[i].op == right_prefix[i].op by {
            assert(left_prefix[i] == left[i]);
            assert(right_prefix[i] == right[i]);
        }
        lookup_is_kind_agnostic(left_prefix, right_prefix, target);
        assert(left.last().op == right.last().op);
        if frame_matches(left.last(), target) {
            assert(frame_matches(right.last(), target));
        } else {
            assert(!frame_matches(right.last(), target));
        }
    }
}

} // verus!
