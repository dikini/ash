# TASK-2013 Row-Aware Typed Handler Semantics Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Type all currently parseable computation-row forms for source handlers, including exact residual rows, handler answer types, and affine versus multi-shot continuations, without making handlers executable.

**Architecture:** Add a structural computation-row normalizer, a partial deterministic row-union
operation, and an immutable `CheckedComputation` inference boundary in `ash-typeck`.  It derives
`result_type`, normalized row, expression anchor, and per-effect provenance directly from the
existing expression AST.  A declared concrete operation call contributes its singleton row; an
explicitly audited pure composite unions only its recursively inferred children; existing
row-bearing declarations/annotations are normalized and unified with the inferred row.  Every
other expression form rejects fail-closed instead of being assumed pure.  Canonical `on`
declarations and `handle expr with h` consume this same boundary.  The latter implicitly supplies
the source thunk `Unit -> {inferred row} result` and unifies it exactly with the handler input.
Alias/group expansion is structural and fail-closed. The existing Core inspection bridge remains
narrow and nonproduction because Core lacks a general multi-clause/done carrier.

**Tech Stack:** Rust 2024; `ash-parser` surface AST; `ash-typeck`; existing declared concrete operation resolver; `proptest` where row-normalization invariants can be generated.

**Authority:** [TASK-2013](../plan/tasks/TASK-2013-source-handler-and-handle-lowering.md), [SPEC-097b §8.8](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#88-handler-typing), [SPEC-096b](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md), and [SPEC-098c §6](../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md#6-handlers-and-provider-boundaries).

---

### Task 1: Write failing all-row-form normalization tests

**Files:**

- Create: `crates/ash-typeck/tests/task_2013_row_normalization.rs`
- Modify: `crates/ash-typeck/src/lib.rs` (test-only/public-to-crate seam only if required)

**Step 1: Write tests** for direct concrete operations, aliases, groups, an open tail, and every
currently parseable non-operation family. Assert structural alias/group expansion, preserved tail,
source-visible provenance, and no loss of retained items.

**Step 2: Write failure controls** for alias/group cycles, unknown concrete operations, malformed or
private imported row summaries, and conflicting tails. Assert no handler fact is published and no
hidden dependency identifier reaches diagnostics.

**Step 3: Run RED.**

Run: `cargo test -p ash-typeck --test task_2013_row_normalization -- --nocapture`

Expected: FAIL because no general structural normalizer exists.

### Task 2: Implement the structural normalizer

**Files:**

- Create: `crates/ash-typeck/src/handler_rows.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2013_row_normalization.rs`

**Step 1: Implement immutable normalized facts** for items, optional tail, source anchors, and
expansion provenance. Walk `ComputationRow` and existing row-definition/type-environment data
directly. Expand aliases/groups with an explicit recursion stack and resolve operations through
`resolve_declared_concrete_operation`.

**Step 2: Make every unavailable expansion fail closed.** Never replace a failed reference, unknown
summary, cycle, privacy boundary, or unsupported item with an empty row.

**Step 3: Run GREEN.**

Run: `cargo test -p ash-typeck --test task_2013_row_normalization -- --nocapture`

Expected: PASS.

**Step 4: Add a property test** generating permutations of distinct supported normalized items.
Normalization must be deterministic, preserve the item set/tail, and never invent an operation.

### Task 3: Write failing implicit-thunk computation-inference tests

**Files:**

- Create: `crates/ash-typeck/tests/task_2013_checked_computation_inference.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/lib.rs` (test-only/public-to-crate seam only if required)

**Step 1: Write inference tests.** A declared concrete `ImplType::operation(args...)` call must
produce an immutable computation fact with its declared result type, a singleton concrete
operation row, and anchors for both the call and operation spelling. Test supported pure
composition around such calls (literal/resolved-value/grouping, tuple/collection/record,
unary/binary operator, and existing branch/sequence forms whose children are inferable): their
rows are the deterministic union of child rows.

**Step 2: Write row-source and failure controls.** Feed declared computation/thunk annotations
and signatures containing aliases, groups, every non-operation item, and open tails through the
same normalizer. Assert annotation-vs-inference mismatch names both source anchors, equal tails
merge, conflicting tails reject, and operation provenance is retained after union. Assert a
generic (non-declared-operation) callable application, assignment, unclassified control/runtime
form, macro, and one deliberately
unclassified AST form fail at their own anchor with
`unsupported-handler-computation-expression`; none may acquire an empty row by default.

**Step 3: Run RED.**

Run: `cargo test -p ash-typeck --test task_2013_checked_computation_inference -- --nocapture`

Expected: FAIL because the checker has no immutable expression-to-computation inference boundary.

### Task 4: Implement checked-computation inference and structural row union

**Files:**

- Create or modify: `crates/ash-typeck/src/handler_rows.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Test: `crates/ash-typeck/tests/task_2013_checked_computation_inference.rs`

**Step 1: Define immutable facts and one inference entry point.** Add
`CheckedComputation { result_type, normalized_row, expression_anchor, effect_anchors }` plus a
single AST-directed inference entry point. Do not use printed-row text, Core inspection, or a
runtime closure. Reuse ordinary expression type facts only where their AST form has been
explicitly classified as pure by this task.

**Step 2: Define `union_rows` before composition.** Normalize each input first; canonicalize
concrete operations by declared identity while retaining all source provenance, retain
non-operation items by normalized structural identity, and permit at most one equal resolved
tail. Distinct/unresolved tails and any failed normalization reject with relevant anchors.
Document and test deterministic ordering, idempotence for identical contributions, and compatible
associativity. Union is not a subset relation and may not discard an item.

**Step 3: Classify expression forms fail-closed.** Implement declared concrete operation calls
as a qualified special case before rejecting generic calls, and implement the audited pure
composites from Task 3. Union supported argument/child rows; a concrete operation call then adds
its singleton operation row. Normalize/unify existing annotation and
signature rows, retaining aliases/groups/open tails/non-operation source items. Route every
unclassified form to the stable unsupported-computation diagnostic rather than `{}`.

**Step 4: Run GREEN and regression controls.**

Run: `cargo test -p ash-typeck --test task_2013_checked_computation_inference -- --nocapture && cargo test -p ash-typeck --test task_2013_row_normalization -- --nocapture`

Expected: PASS.

### Task 5: Write failing residual, answer, and multiplicity tests

**Files:**

- Create: `crates/ash-typeck/tests/task_2013_handler_row_typing.rs`
- Modify: `crates/ash-typeck/src/lib.rs`

**Step 1: Write declaration tests** proving a canonical `on` peels each handled operation exactly
once; retains resource, role, policy, contract, channel, process, failure, evidence, unhandled
alias/group expansion, and open tails; rejects duplicate/absent clauses; binds `done(value)` as
the handled result `A`; and makes all branches share `Ans`.

**Step 2: Add multiplicity controls.** Repeated `resume` succeeds only with a normalized
closed-empty residual; it rejects with every nonempty residual and every open tail.

**Step 3: Run RED.**

Run: `cargo test -p ash-typeck --test task_2013_handler_row_typing -- --nocapture`

Expected: FAIL because the sidecar has no general residual fact and recognizes only direct affine
resume calls.

### Task 6: Implement typed handler-declaration facts

**Files:**

- Modify: `crates/ash-typeck/src/lib.rs:check_handler_declarations`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/handler_rows.rs`
- Test: `crates/ash-typeck/tests/task_2013_handler_row_typing.rs`
- Test: `crates/ash-typeck/tests/task_2013_checked_handler_declaration.rs`

**Step 1: Replace the direct-resume special case.** Invoke the Task-4 AST-directed
`CheckedComputation` inference for the canonical `on computation` operand. Normalize its row,
compute `r = R - H`, and bind each continuation at
`B_op -> {r} Ans` with the exact multiplicity.

**Step 2: Publish transactionally.** Do not register a checked handler after normalization,
subtraction, answer, or multiplicity failure. Preserve exact source anchors in the final facts.

**Step 3: Run GREEN and regressions.**

Run: `cargo test -p ash-typeck --test task_2013_handler_row_typing -- --nocapture && cargo test -p ash-typeck --test task_2013_checked_handler_declaration -- --nocapture`

Expected: PASS.

### Task 7: Write failing implicitly-thunked `handle expr with h` tests

**Files:**

- Create: `crates/ash-typeck/tests/task_2013_handle_with_row_typing.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`

**Step 1: Write tests** for implicit thunk construction from a declared concrete operation
expression, pure composition around it, alias/group-expanded annotation/signature rows, open-tail
input, answer/output residual facts, ordinary-function rejection, wrong marker, and mismatched
result/row input. Assert no source thunk syntax or runtime closure is required.

**Step 2: Add fail-closed controls.** A generic callable application, assignment, unclassified
control/runtime form, macro, and an unclassified expression must reject through the computation-inference
diagnostic before handler unification. An incompatible normalized tail or row annotation must
identify the expression and declaration anchors without leaking private dependencies.

**Step 3: Assert boundary controls.** Success must neither call
`lower_checked_handler_application_to_core` nor create an engine/provider/handler-frame artifact.

**Step 4: Run RED.**

Run: `cargo test -p ash-typeck --test task_2013_handle_with_row_typing -- --nocapture`

Expected: FAIL because current `HandleWith` has no implicit AST-directed computation inference or
exact normalized thunk-row unification.

### Task 8: Implement typed `handle expr with h` checking

**Files:**

- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/handler_rows.rs`
- Test: `crates/ash-typeck/tests/task_2013_handle_with_row_typing.rs`

**Step 1: Infer, resolve, then verify.** First invoke the sole Task-4 `CheckedComputation`
inference entry point for the source `expr`; interpret the resulting immutable fact as the
implicit `Unit -> {R_expr} A_expr` input. Resolve `h` in the value namespace, require its handler
marker, retrieve its checked fact, and unify it exactly with `Unit -> {R_handler} A_handler`.
Produce a typed application fact with answer type, residual output row, expression/handler anchors,
and preserved row provenance.

**Step 2: Keep inference and execution boundaries explicit.** Implicit thunking is a type fact,
not a source rewrite, runtime closure, or fallback. Propagate the stable unsupported-expression,
normalization, or exact-unification diagnostic before marker/lowering fallback. Do not treat
`Expr::On` as runtime installation.

**Step 3: Run GREEN.**

Run: `cargo test -p ash-typeck --test task_2013_handle_with_row_typing -- --nocapture`

Expected: PASS.

### Task 9: Protect Core/CPS and runtime boundaries

**Files:**

- Modify: `crates/ash-typeck/tests/task_2013_handler_core_lowering.rs`
- Create or modify: `crates/ash-engine/tests/task_2013_source_handler_runtime_boundary.rs`

**Step 1: Write boundary tests.** General multi-clause, nonidentity-`done`, open-tail, and
multi-shot typed facts must reject at the narrow inspection bridge rather than downgrade or execute.
Existing compatible inspection fixtures retain their exact `Handle`/`Raise` shape.

**Step 2: Apply only minimal bridge gating** if tests expose accidental admission. Do not add Core
fields, CPS forms, engine registration, or frame construction.

**Step 3: Run focused controls.**

Run: `cargo test -p ash-typeck --test task_2013_handler_core_lowering -- --nocapture && cargo test -p ash-engine --test task_2013_source_handler_runtime_boundary -- --nocapture`

Expected: PASS; general typed facts stay nonproduction.

### Task 10: QA, review, and evidence

**Files:**

- Modify: `docs/plan/tasks/TASK-2013-source-handler-and-handle-lowering.md` (evidence only after verification)
- Modify: `CHANGELOG.md` only if this delivered scope triggers project completion policy

**Step 1: Run gates.**

Run:

```bash
cargo fmt --check
cargo test -p ash-typeck --test task_2013_row_normalization
cargo test -p ash-typeck --test task_2013_checked_computation_inference
cargo test -p ash-typeck --test task_2013_handler_row_typing
cargo test -p ash-typeck --test task_2013_handle_with_row_typing
cargo test -p ash-typeck --test task_2013_checked_handler_declaration
cargo test -p ash-typeck --test task_2013_handler_core_lowering
cargo test -p ash-engine --test task_2013_source_handler_runtime_boundary
cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
python3 tools/docs/validate_semantic_traceability.py --root . --graph docs/spec/SEMANTIC-TRACEABILITY.json
git diff --check
```

**Step 2: Require independent QA and code review** of preservation, privacy/cycle fail-closed
behavior, continuation multiplicity, transactional publication, and the no-runtime boundary.

**Step 3: Record exact evidence.** Keep TASK-2013 **in progress**: general multi-clause/done
Core/CPS lowering and runtime behavior remain outside this slice. Do not commit without user
authorization.

## Completion definition for this slice

Every currently parseable row form is structurally normalized and accepted or rejected
deterministically; immutable expression-derived computations use fail-closed, AST-directed
inference and deterministic row union; implicit source thunking makes `handle expr with h` consume
that fact rather than an undeclared external fact; handlers get exact residual rows and correct
continuation multiplicity; aliases/groups/open tails and non-operation families are not erased;
and no production Core/CPS lowering, handler/provider authority, or runtime execution has been
added.
