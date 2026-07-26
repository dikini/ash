# TASK-2013 Row-Aware Typed Handler Semantics Design

**Status:** Approved implementation design

**Authority:** [TASK-2013](../plan/tasks/TASK-2013-source-handler-and-handle-lowering.md), [SPEC-097b §8.8](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#88-handler-typing), [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md), [SPEC-098c §6](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md#6-handlers-and-provider-boundaries), and [SPEC-100 §11.10](../spec/SPEC-100-CORE-TYPE-CHECKING.md#1110-handle).

## Decision

Add a **row-aware typed-computation layer** in `ash-typeck`. It consumes the structural
`ComputationRow` AST and declaration-backed operation resolver directly; it never reparses
printed row text. It produces immutable normalized-row, typed-handler-clause, and
typed-handler-application facts. Those facts are requirements only: they grant no provider,
handler frame, dispatch right, runtime authority, or execution behavior.

`handle expr with h` uses **implicit thunking and inference**.  Before handler lookup, the
typechecker constructs an immutable `CheckedComputation` directly from the existing expression
AST:

```text
CheckedComputation {
    result_type: A,
    normalized_row: R,
    expression_anchor: source origin/span,
    effect_anchors: immutable provenance for every contributed row item/tail,
}
```

This is not a runtime closure and does not rewrite the source AST.  It is the typed evidence for
the implicit source thunk `Unit -> {R} A`.  `handle expr with h` then resolves `h` in the value
namespace, requires its handler marker, and unifies that inferred thunk exactly with the handler
input.  The row comparison is structural after normalization, including the open-tail identity;
it is not a subset check and it never reparses printed rows.  Success records a typed application
fact anchored both at `expr` and at the handler name.

Inference has a deliberately finite, fail-closed expression boundary.  It supports declared
concrete `ImplType::operation(args...)` calls (the call contributes that operation's singleton
row, anchored at its qualified operation spelling), and audited ordinary pure forms composed from
already-inferred children: literals, resolved values, grouping, tuples/collections/records,
unary/binary pure operators, and the existing sequencing/branch forms when every control and
result child is inferable.  A pure form contributes the structural union of its children; it is
not assumed pure merely because it has no row annotation. Generic callable applications,
assignment, unclassified control/runtime forms, macros, or any AST variant not explicitly
classified by this stage reject
with a stable `unsupported-handler-computation-expression` diagnostic at its source anchor.
They must never be silently assigned `{}`.  Existing declared computation/thunk annotations and
row-bearing declaration signatures feed their `ComputationRow` through the same normalizer, so
aliases, diagnostic groups, non-operation items, and open tails participate without text parsing.
A plain function, non-computation expression, failed inference, or incompatible thunk row is a
deterministic type error, not a fall-through to the private Core inspection bridge.

`on computation { ... }` uses the identical inference boundary for `computation`; it publishes a
handler fact only after its inferred `CheckedComputation` has been normalized, subtracted, and
answer-checked.  Thus the source spelling `on expr` and the `handle expr with h` operand have one
meaning, rather than an undeclared "typed computation fact" supplied by a later lowering pass.

## Normalization and subtraction

The normalizer accepts every currently parseable family: concrete operations, aliases/whole-row
references, diagnostic groups, resource, role, policy, contract, channel, process, failure,
evidence, and one open tail. It resolves concrete operations through the declared resolver;
expands aliases structurally with a recursion stack; expands groups while retaining the group name
and source anchor as diagnostic provenance; preserves all non-operation items and the optional tail;
and rejects cycles, inaccessible/private dependencies, malformed imported summaries, duplicate or
conflicting tails, and unsupported expansion states before publishing a fact.

Subtraction removes only one matching normalized concrete operation for each distinct source clause.
It does not remove a non-operation item, group provenance, or tail. Duplicate source clauses reject
before subtraction. Alias/group-expanded duplicate operations canonicalize with provenance retained.
`closed_empty` means exactly: no normalized items and no tail. Privacy/cycle/unresolved failures are
fail-closed and diagnostics must not expose inaccessible dependency names.

## Row union and composition

Expression inference and clause-body checking use one partial structural `union_rows` operation.
It first normalizes every source row, then merges canonical concrete-operation identities while
retaining every contributing source anchor/provenance; identical contributions do not create a
second semantic operation.  Non-operation items are retained under their normalized structural
identity rather than discharged or converted to operations.  There may be at most one open tail:
equal resolved tail identities merge with both anchors retained, and distinct or unresolved tails
reject at the two relevant source anchors.  Union must be deterministic (canonical item order plus
source-order provenance), associative for compatible inputs, and idempotent for an identical
contribution.  It is not effect subtyping, a provider grant, or a permission to erase an
unrecognized item.

For a declared concrete operation call, inference unions the rows of all supported argument
expressions with the singleton operation row.  For a supported ordinary pure composite, it unions
only child rows.  A source annotation/signature row is normalized and unified with the inferred
row at that annotation's own anchor; disagreement rejects rather than overwriting either source.
The handler input unification consequently compares `Unit -> {R_expr} A_expr` against
`Unit -> {R_handler} A_handler` with anchors for the expression, handler declaration, every row
item, and any tail available to diagnostics.

## Handler rules

For canonical `on computation { op_i(payload_i, k_i) => body_i; done(value) => done_body }`, let
the typed computation have result `A`, normalized row `R`, handled operations `H`, and
`r = R - H`. Every `op_i` must be present exactly once in `R` after normalization. If the declared
result of `op_i` is `B_i`, bind `k_i : B_i -> {r} Ans`.

- `k_i` is `MultiShotPure` iff `r` is closed empty; otherwise, including an open tail, it is
  `Affine` and may be called at most once.
- Every operation body and `done` body has the shared answer type `Ans`.
- The `done` binder has the handled computation result type `A`.
- The resulting typed computation has answer type `Ans` and row `r` union checked clause-body
  effects using the normal row-union rule. Ambient non-operation items are never discharged.

`check_handler_declarations` consumes this layer for canonical `on` bodies and publishes facts only
after all checks succeed. `Expr::HandleWith` consumes the same facts. `Expr::On` remains
declaration-only absent a separately approved general source-expression rule.

## Existing bridge and explicit deferrals

The existing `lower_checked_handler_application_to_core` remains a private inspection control. It
may accept only its already-supported narrow shape and must explicitly reject general multi-clause,
arbitrary-`done`, open-tail, and multi-shot typed facts. Core lacks the required general
multi-clause/done carrier, so no production Core/CPS lowering is authorized here.

This design excludes engine registration, handler/provider frames, dispatch, execution, host
authority, cancellation/timeouts, provider inference, textual row reparse, legacy `invoke`,
historical handler syntax, and cross-module handler execution/admission.

## Evidence required

Tests must cover concrete-operation expression inference, supported pure composition, explicit
row-bearing alias/group/open-tail/non-operation inputs, unsupported-expression fail-closed
diagnostics, exact row union/source anchors, exact subtraction, cycle/privacy fail-closed
diagnostics, closed-empty versus nonempty/open continuation multiplicity, typed implicit-thunk
`handle expr with h`, and the continued no-Core/no-runtime boundary.
