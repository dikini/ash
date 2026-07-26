# TASK-2013 Canonical `on` Grammar Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Enforce the approved `on` clause cardinality—at least one operation and exactly one `done`—without changing handler execution or lowering.

**Architecture:** The repository already has the source-preserving `Expr::On` and
`HandlerClause` carriers, keyword support for `on`/`done`, and a `parse_on_expr` route before
ordinary identifier parsing. Add the source boundary checks in that existing parser and repeat
them in `check_handler_declarations`, the path that produces checked declaration facts. The
general expression checker's direct `Expr::On` branch remains explicitly unsupported and is not
a meaningful guard for arbitrary handler facts. Do not add Core/CPS, residual-row, or runtime
behavior.

**Tech Stack:** Rust 2024, `ash-parser`, `ash-typeck`, Cargo.

**Authority:** SPEC-095b §4.3; TASK-2013; the approved bounded slice. `ImplType::operation`
remains the normal symbolic operation form, but duplicate-operation semantics are not approved
in this work.

---

### Task 1: Parser cardinality tests

**Files:**
- Modify: `crates/ash-parser/tests/task_2013_handler_surface.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs:249-329`

**Step 1: Write failing parser tests.**

Extend the existing TASK-2013 surface suite with source fixtures for:

- a valid `on` body with one operation and one `done`;
- an `on` body containing only `done`, rejected as missing an operation;
- an `on` body with an operation but no `done`, rejected as missing `done`; and
- an `on` body with two `done` clauses, rejected at the second `done`.

Assert deterministic, source-oriented diagnostics/spans. Keep a valid repeated concrete
operation fixture admissible or otherwise leave it unasserted: duplicate concrete-operation
semantics are explicitly out of scope.

**Step 2: Prove red.**

Run: `cargo test -p ash-parser --test task_2013_handler_surface -- --nocapture`

Expected: FAIL because the existing `parse_on_expr` collects clauses without enforcing these
cardinalities.

**Step 3: Implement the minimal parser checks.**

In the existing `parse_on_expr`, track whether at least one `HandlerClause::Operation` was
parsed and whether a `HandlerClause::Done` has already appeared. Reject a second `done` when its
keyword is encountered; after the closing brace, reject zero operations or absent `done`. Use the
existing parser error conventions and source positions. Do not add lexer/token work, change
`Expr::On` or `HandlerClause`, canonicalize operation names, or introduce an operation-duplicate
check.

**Step 4: Prove green.**

Run: `cargo test -p ash-parser --test task_2013_handler_surface -- --nocapture`

Expected: PASS, including existing source-carrier tests and new deterministic cardinality cases.

### Task 2: Checked-handler declaration cardinality tests

**Files:**
- Modify: `crates/ash-typeck/tests/task_2013_checked_handler_declaration.rs`
- Modify: `crates/ash-typeck/src/lib.rs:1121-1240` (`check_handler_declarations`)

**Step 1: Write failing type-checker tests.**

Use parsed handler declarations in the existing TASK-2013 checked-handler suite to verify the
declaration checker rejects zero operation clauses and rejects a second `done` deterministically.
Use a parser-valid constructed/fixture route only if needed to reach the checker after Task 1;
do not test through the general expression checker's unsupported `Expr::On` path. Retain the
existing missing-`done` declaration assertion and make its diagnostic stable.

**Step 2: Prove red.**

Run: `cargo test -p ash-typeck --test task_2013_checked_handler_declaration -- --nocapture`

Expected: FAIL because `check_handler_declarations` presently accepts an empty `operations`
vector while it only detects duplicate or missing `done` clauses.

**Step 3: Implement the minimal declaration check.**

After traversing clauses in `check_handler_declarations`, reject an empty operation collection
with the same stable subject as the parser. Retain the existing single-`done` check; adjust it
only as needed so a second `done` is deterministically rejected before declaration facts are
published. Do not add duplicate concrete-operation identity checks, change normal `check_expr`
handling of `Expr::On`, type general handler semantics, or alter the bounded inspection bridge.

**Step 4: Prove green.**

Run: `cargo test -p ash-typeck --test task_2013_checked_handler_declaration -- --nocapture`

Expected: PASS, including existing operation-resolution, answer-type, handler-marker, and direct
resume checks.

### Task 3: Regression, review, and bounded verification

**Files:**
- Modify: `docs/plan/tasks/TASK-2013-source-handler-and-handle-lowering.md` only after implementation results are known

**Step 1: Run focused regression suites.**

Run:

```bash
cargo test -p ash-parser --test task_2013_handler_surface
cargo test -p ash-typeck --test task_2013_checked_handler_declaration
cargo fmt --check
cargo clippy -p ash-parser -p ash-typeck --all-targets --all-features -- -D warnings
```

Expected: PASS. Record any independent pre-existing failure with its command and output; do not
present it as TASK-2013 success.

**Step 2: Review scope.**

Review the parser and declaration-checker changes to confirm that they alter only structural
cardinality. Confirm no lexer/carrier churn, duplicate-operation rule, Core term, CPS term,
runtime handler frame, residual-row behavior, or `invoke` fallback was added.

**Step 3: Document only verified evidence.**

Record exact verified results in the existing TASK-2013 record. Do not change task or phase
status, or `CHANGELOG.md`, unless the task's whole completion criteria are met.

## Execution evidence

The bounded plan was executed without widening its authority.  Parser enforcement rejects an
`on` body with no operation clause or no `done`, and rejects a second `done` at that second
keyword's source position.  The checked-handler declaration path repeats the guard for
constructed ASTs, rejecting zero operations, missing `done`, and duplicate `done` before it
publishes checked-handler facts.

Verified results: the parser fixture suite passed 6/6; the checked-handler declaration suite
passed 8/8; the focused relevant QA set passed 26 tests; full parser and typechecker suites,
formatting, affected-crate warnings-denied Clippy, and documentation gates passed.  This is not
whole TASK-2013 completion: no duplicate-operation rule, general handler semantics, Core/CPS
production lowering, provider/handler frame construction, or runtime execution was added.
