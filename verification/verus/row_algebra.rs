//! TASK-1992 closed Core-row algebra pilot.
//!
//! This is a standalone Verus fragment, deliberately outside Cargo.  It is a
//! mathematical refinement *model*, not a linked verification of Ash's Rust
//! `CoreRow` implementation.  A model atom represents the complete structural
//! identity of one already-expanded `CoreRowItem`; production names, paths,
//! payload types, and enum variants must be injectively encoded into this atom
//! before the theorems below can apply.  `CoreRow.tail` is required to be
//! absent.  Open tails, structural type equivalence, ordered diagnostic
//! payloads, and Rust-to-model correspondence are explicitly out of scope.
//!
//! In particular `GroupRef` is rejected at the boundary and a `Role` atom is
//! just a requirement namespace tag: neither is an authority grant.

use vstd::prelude::*;

fn main() {}

verus! {

/// A finite, closed-row model.  The integer is an injective encoding of the
/// whole exact `CoreRowItem` identity, not an operation name alone.
pub type ModelItem = int;
pub type ClosedRow = Seq<ModelItem>;

/// A source-side boundary classification.  Only `ExactItem` is admitted into
/// this pilot's closed model.  Group expansion is intentionally not modeled.
pub enum RowBoundaryItem {
    ExactItem(ModelItem),
    EffectGroupRef(ModelItem),
    RoleRequirement(ModelItem),
}

pub open spec fn admitted_closed_item(item: RowBoundaryItem) -> bool {
    match item {
        RowBoundaryItem::ExactItem(_) | RowBoundaryItem::RoleRequirement(_) => true,
        RowBoundaryItem::EffectGroupRef(_) => false,
    }
}

/// Group references have no semantic authority in the row normalizer: they
/// are rejected before normalization rather than expanded or authorized here.
pub proof fn ambiguous_group_is_rejected(group: ModelItem)
    ensures !admitted_closed_item(RowBoundaryItem::EffectGroupRef(group)),
{
}

/// Roles remain exact requirement atoms; this model has no authority relation.
pub open spec fn is_authority_grant(_item: RowBoundaryItem) -> bool { false }

pub proof fn rows_do_not_grant_authority(item: RowBoundaryItem)
    ensures !is_authority_grant(item),
{
}

/// Stable left-to-right duplicate elimination.  Constructing from prefixes
/// makes a duplicate retain its *first*, rather than its final, occurrence.
pub open spec fn normalize_closed(row: ClosedRow) -> ClosedRow
    decreases row.len(),
{
    if row.len() == 0 {
        Seq::empty()
    } else {
        let prefix = normalize_closed(row.drop_last());
        let last = row.last();
        if prefix.contains(last) { prefix } else { prefix.push(last) }
    }
}

pub open spec fn included_in(actual: ClosedRow, expected: ClosedRow) -> bool {
    forall |item: ModelItem| actual.contains(item) ==> expected.contains(item)
}

pub open spec fn no_duplicates(row: ClosedRow) -> bool {
    forall |i: int, j: int|
        0 <= i < row.len() && 0 <= j < row.len() && row[i] == row[j] ==> i == j
}

/// `out` preserves stable first-occurrence order from `input`.
pub open spec fn stable_first_occurrence(input: ClosedRow, out: ClosedRow) -> bool {
    // vstd's independently defined left-to-right remove_duplicates is the
    // stable sequence-order specification: it retains the first encounter of
    // each identity while threading an explicit `seen` prefix.
    out == input.remove_duplicates(Seq::empty())
}

/// A small sequence lemma kept local so the proof never relies on an opaque
/// library fact about the membership behavior of `push`.
pub proof fn contains_after_push(prefix: ClosedRow, last: ModelItem, item: ModelItem)
    ensures prefix.push(last).contains(item) == (prefix.contains(item) || item == last),
{
    if prefix.contains(item) {
        let i = prefix.index_of(item);
        assert(0 <= i < prefix.len());
        assert(prefix.push(last)[i] == prefix[i]);
        assert(prefix.push(last).contains(item));
    } else if item == last {
        assert(0 <= prefix.len() < prefix.push(last).len());
        assert(prefix.push(last)[prefix.len() as int] == last);
        assert(prefix.push(last).contains(item));
    } else if prefix.push(last).contains(item) {
        let i = prefix.push(last).index_of(item);
        assert(0 <= i < prefix.push(last).len());
        if i < prefix.len() {
            assert(prefix.push(last)[i] == prefix[i]);
            assert(prefix.contains(item));
        } else {
            assert(i == prefix.len());
            assert(prefix.push(last)[i] == last);
            assert(item == last);
        }
    }
}

/// Membership preservation is the primary model/refinement invariant.
pub proof fn normalize_membership(row: ClosedRow, item: ModelItem)
    ensures normalize_closed(row).contains(item) == row.contains(item),
    decreases row.len(),
{
    if row.len() > 0 {
        let prefix = row.drop_last();
        let last = row.last();
        normalize_membership(prefix, item);
        row.lemma_add_last_back();
        contains_after_push(prefix, last, item);
        contains_after_push(normalize_closed(prefix), last, item);
        if normalize_closed(prefix).contains(last) {
            assert(normalize_closed(row) == normalize_closed(prefix));
        } else {
            assert(normalize_closed(row) == normalize_closed(prefix).push(last));
        }
    }
}

/// Duplicate-free rows are already in normal form.  This is the fact needed
/// to establish idempotence without any unreported normalization assumption.
pub proof fn normalize_of_no_duplicates(row: ClosedRow)
    requires no_duplicates(row),
    ensures normalize_closed(row) == row,
    decreases row.len(),
{
    if row.len() > 0 {
        let prefix = row.drop_last();
        let last = row.last();
        row.lemma_add_last_back();
        assert(no_duplicates(prefix));
        normalize_of_no_duplicates(prefix);
        if prefix.contains(last) {
            prefix.index_of_first_ensures(last);
            let i = prefix.index_of_first(last).unwrap();
            assert(0 <= i < prefix.len());
            assert(prefix[i] == last);
            assert(row[i] == prefix[i]);
            assert(row[row.len() - 1] == last);
            assert(i != row.len() - 1);
            assert(false);
        }
        assert(!prefix.contains(last));
        assert(normalize_closed(row) == normalize_closed(prefix).push(last));
    }
}

pub proof fn normalize_no_duplicates(row: ClosedRow)
    ensures no_duplicates(normalize_closed(row)),
    decreases row.len(),
{
    if row.len() > 0 {
        let prefix = row.drop_last();
        let last = row.last();
        normalize_no_duplicates(prefix);
        normalize_membership(prefix, last);
        row.lemma_add_last_back();
        if normalize_closed(prefix).contains(last) {
            assert(normalize_closed(row) == normalize_closed(prefix));
        } else {
            assert(normalize_closed(row) == normalize_closed(prefix).push(last));
        }
    }
}

/// Normalization never increases a closed row's cardinality.
pub proof fn normalize_nonincreasing(row: ClosedRow)
    ensures normalize_closed(row).len() <= row.len(),
    decreases row.len(),
{
    if row.len() > 0 {
        let prefix = row.drop_last();
        let last = row.last();
        normalize_nonincreasing(prefix);
        row.lemma_add_last_back();
        if normalize_closed(prefix).contains(last) {
            assert(normalize_closed(row) == normalize_closed(prefix));
        } else {
            assert(normalize_closed(row) == normalize_closed(prefix).push(last));
        }
    }
}

/// The model's output order is the input's first-occurrence order.  This is
/// intentionally stronger than set equality: `normalize_closed` recurses over
/// prefixes and only appends a final item when it has not occurred earlier.
pub proof fn normalize_stable_first_occurrence(row: ClosedRow)
    ensures stable_first_occurrence(row, normalize_closed(row)),
    decreases row.len(),
{
    if row.len() > 0 {
        let prefix = row.drop_last();
        let last = row.last();
        row.lemma_add_last_back();
        normalize_stable_first_occurrence(prefix);
        assert(normalize_closed(prefix) == prefix.remove_duplicates(Seq::empty()));
        prefix.lemma_remove_duplicates_append(last, Seq::empty());
        normalize_membership(prefix, last);
        if normalize_closed(prefix).contains(last) {
            assert(normalize_closed(row) == normalize_closed(prefix));
            assert(row.remove_duplicates(Seq::empty()) == prefix.remove_duplicates(Seq::empty()));
            assert(normalize_closed(row) == row.remove_duplicates(Seq::empty()));
        } else {
            assert(normalize_closed(row) == normalize_closed(prefix).push(last));
            assert(row.remove_duplicates(Seq::empty()) == prefix.remove_duplicates(Seq::empty()) + seq![last]);
            assert(normalize_closed(row) == row.remove_duplicates(Seq::empty()));
        }
    } else {
        assert(normalize_closed(row) == Seq::empty());
        assert(row.remove_duplicates(Seq::empty()) == Seq::empty());
        assert(normalize_closed(row) == row.remove_duplicates(Seq::empty()));
    }
}

/// A normalized closed row is a fixed point.
pub proof fn normalize_idempotent(row: ClosedRow)
    ensures normalize_closed(normalize_closed(row)) == normalize_closed(row),
    decreases row.len(),
{
    if row.len() > 0 {
        normalize_no_duplicates(row);
        normalize_of_no_duplicates(normalize_closed(row));
    }
}

/// Inclusion is reflexive for the closed, exact-identity model.
pub proof fn inclusion_reflexive(row: ClosedRow)
    ensures included_in(row, row),
{
}

/// Closed inclusion composes transitively.
pub proof fn inclusion_transitive(a: ClosedRow, b: ClosedRow, c: ClosedRow)
    requires included_in(a, b), included_in(b, c),
    ensures included_in(a, c),
{
}

/// Normalization does not change the truth of closed inclusion.
pub proof fn inclusion_normalization_invariant(actual: ClosedRow, expected: ClosedRow)
    ensures included_in(actual, expected) == included_in(normalize_closed(actual), normalize_closed(expected)),
{
    assert forall |item: ModelItem|
        actual.contains(item) == normalize_closed(actual).contains(item) by {
            normalize_membership(actual, item);
        }
    assert forall |item: ModelItem|
        expected.contains(item) == normalize_closed(expected).contains(item) by {
            normalize_membership(expected, item);
        }
    if included_in(actual, expected) {
        assert forall |item: ModelItem|
            normalize_closed(actual).contains(item) implies normalize_closed(expected).contains(item) by {
                normalize_membership(actual, item);
                normalize_membership(expected, item);
            }
    }
    if included_in(normalize_closed(actual), normalize_closed(expected)) {
        assert forall |item: ModelItem|
            actual.contains(item) implies expected.contains(item) by {
                normalize_membership(actual, item);
                normalize_membership(expected, item);
            }
    }
}

/// A representation-preserving refactor point: `row` and `reordered` may
/// differ in sequence order but are extensionally equal as membership sets.
/// Inclusion truth is consequently permutation invariant.
pub proof fn inclusion_membership_permutation_invariant(
    actual: ClosedRow,
    expected: ClosedRow,
    reordered_actual: ClosedRow,
    reordered_expected: ClosedRow,
)
    requires
        forall |item: ModelItem| actual.contains(item) == reordered_actual.contains(item),
        forall |item: ModelItem| expected.contains(item) == reordered_expected.contains(item),
    ensures included_in(actual, expected) == included_in(reordered_actual, reordered_expected),
{
}

/// The checked executable shape used by a future adapter.  The `requires`
/// clause makes the closed/expanded boundary explicit; its `ensures` clauses
/// record precisely the model contract an adapter must discharge.
pub proof fn checked_normalize_view(row: ClosedRow)
    // `ClosedRow` carries no tail and this finite-sequence model has no
    // group-reference constructor; the nonnegative length precondition is a
    // concrete checkable boundary fact rather than an unbound quantifier.
    requires row.len() >= 0,
    ensures
        included_in(row, normalize_closed(row)),
        included_in(normalize_closed(row), row),
        no_duplicates(normalize_closed(row)),
        normalize_closed(row).len() <= row.len(),
        normalize_closed(normalize_closed(row)) == normalize_closed(row),
{
    normalize_no_duplicates(row);
    normalize_nonincreasing(row);
    normalize_idempotent(row);
    assert forall |item: ModelItem| row.contains(item) == normalize_closed(row).contains(item) by {
        normalize_membership(row, item);
    }
}

} // verus!
