# TASK-2013 `on expr` Stop-Set Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Parse the full existing expression grammar after `on` while recognizing the subsequent
canonical handler-clause brace as a parser-local delimiter.

**Architecture:** Keep `Expr::On`, `HandlerClause`, clause parsing, and cardinality checks intact.
Refactor only the expression entry path needed by `parse_on_expr` so it can invoke the normal
precedence grammar in an `on`-computation mode.  The mode adds a non-consuming clause-shaped
brace lookahead at the named inline-record suffix decision; all nested expression parsing stays in
normal mode.  Parsing remains structural and fail closed at the existing typed/lowering boundary.

**Tech Stack:** Rust 2024; `ash-parser`; existing TASK-2013 parser, typechecker, and Core
inspection tests; Cargo.

**Scope authority:** [approved design](2026-07-24-task-2013-on-expression-stop-set-design.md),
SPEC-095b §4.3, and TASK-2013.  This plan does not authorize runtime behavior.

---

### Task 1: Add red parser contracts for general `on` computations

**Files:**
- Modify: `crates/ash-parser/tests/task_2013_handler_surface.rs`
- Read: `crates/ash-parser/src/parse_expr.rs`

**Step 1: Write failing structural tests.**

Extend the existing TASK-2013 surface suite with distinct source fixtures for:

- `on run(req) { ... }`, asserting `Expr::On.computation` is the existing call carrier;
- `on retries + 1 { ... }`, asserting the existing binary-expression carrier and operand spans;
- `on { request: run(req) } { ... }`, asserting a structural record computation; and
- `on Result { value: run(req) } { ... }`, asserting a named record constructor computation and
  that the second brace, not the record brace, begins handler clauses.

Use one canonical `ImplType::operation(pattern, resume)` clause plus one `done` clause in every
fixture.  Assert `Expr::On` source origin/span and the computation span in addition to its variant;
do not treat parse success alone as evidence.  Include a negative/discriminator control showing a
named record remains a computation before the handler block, and retain the existing cardinality
rejection tests unchanged.

**Step 2: Prove red.**

Run:

```bash
cargo test -p ash-parser --test task_2013_handler_surface -- --nocapture
```

Expected: the new general-computation cases fail because `parse_on_expr` currently calls
`expr_name_with_span` and constructs only `Expr::Variable`.

### Task 2: Introduce a scoped expression parse mode and clause-brace lookahead

**Files:**
- Modify: `crates/ash-parser/src/parse_expr.rs:235-356` (`expr`, `parse_on_expr`)
- Modify: `crates/ash-parser/src/parse_expr.rs:1190-1435` (precedence/primary/postfix route)
- Test: `crates/ash-parser/tests/task_2013_handler_surface.rs`

**Step 1: Add the smallest internal mode carrier.**

Introduce a private expression parse mode (for example, ordinary versus `OnComputation`) and an
internal expression entry function.  Preserve public `expr(input)` as ordinary-mode behavior.
Have `parse_on_expr` invoke the internal entry in `OnComputation` mode after consuming `on`.
Thread that mode through the top-level existing precedence grammar needed to reach a primary and
postfix expression; keep recursive parsing of delimited/nested subexpressions (`parse_args`,
parentheses, record fields, lists, blocks, closures) in ordinary mode.  Do not put mutable global
state in `ParseState`, and do not alter lexer/token definitions.

**Step 2: Define non-consuming delimiter recognition.**

At the existing named inline-record suffix decision after an identifier, add a private lookahead
available only in `OnComputation` mode.  It may suppress the record-suffix parse only when the
current `{` begins, after whitespace/comments, one of these canonical starts:

```text
done (
identifier :: identifier (
```

The lookahead must leave `ParseInput` and its source position unchanged.  It must not call name
resolution or consume a full clause.  Any other brace—including `identifier :` for a record
field—continues through the ordinary inline-record constructor branch.  Once ordinary expression
parsing returns at the recognized brace, leave existing `parse_on_expr` clause parsing and its
cardinality error locations untouched.

**Step 3: Keep malformed behavior committed at the correct boundary.**

A clause-shaped opener followed by a malformed canonical clause is handled by the existing
committed handler-clause parser, not reinterpreted as a record.  A non-clause-shaped brace is
handled by ordinary expression syntax, then the `on` parser still requires its subsequent handler
block.  Do not add error recovery, parenthesis requirements, heuristic operation resolution, or
an `invoke` fallback.

**Step 4: Prove green.**

Run:

```bash
cargo test -p ash-parser --test task_2013_handler_surface -- --nocapture
```

Expected: the existing carrier/cardinality assertions and the new call, binary, record-literal,
and named-record-constructor cases all pass.

### Task 3: Regressions across current handler boundaries

**Files:**
- Test: `crates/ash-parser/tests/task_2013_handler_surface.rs`
- Test: `crates/ash-typeck/tests/task_2013_checked_handler_declaration.rs`
- Test: `crates/ash-typeck/tests/task_2013_handler_core_lowering.rs`

**Step 1: Run parser and existing bounded-handler controls.**

Run:

```bash
cargo test -p ash-parser --test task_2013_handler_surface
cargo test -p ash-typeck --test task_2013_checked_handler_declaration
cargo test -p ash-typeck --test task_2013_handler_core_lowering
```

Expected: PASS.  The typechecker/Core suites remain regression controls; they do not prove that
the newly accepted arbitrary computations have general handler semantics or runtime admission.

**Step 2: Verify scope mechanically.**

Run:

```bash
cargo fmt --check
cargo clippy -p ash-parser --all-targets --all-features -- -D warnings
git diff --check
```

Expected: PASS.  Review the diff to confirm it is parser/test-local: no `ash-engine`, production
Core/CPS, provider/frame, timeout/cancellation, or `invoke` changes.

**Step 3: Review and record only completed evidence.**

Perform a code/spec review against the design invariants.  After verification, update the existing
TASK-2013 record only with the concrete parser evidence and remaining fail-closed boundary.  Do
not mark TASK-2013 complete, update `CHANGELOG.md`, alter plan-index task status, commit, or claim
runtime support: its general row/continuation/Core/CPS/runtime criteria remain open.

---

## Execution Evidence (completed bounded parser slice)

The implementation used a parser-local, non-consuming stop-set only for the top-level computation
following `on`.  It accepts existing call, binary, structural record, and named record constructor
expressions.  The clause-shaped handler opener is recognized only as `done(` or
`identifier::identifier(` after terminal `//`/`--` line-comment or nested block-comment trivia;
the record's first brace is preserved as expression syntax.  Internal comments and quoted marker
text are not delimiter evidence.  Existing clause parsing receives the second brace and retains
the canonical one-or-more-operation/exactly-one-`done` cardinality behavior.

Verified: parser surface 14/14; checked-handler declaration 8/8; Core lowering 9/9; full parser
suite; `cargo fmt --check`; parser Clippy with `-D warnings`; `git diff --check`; and documentation
gates.  This is parser structure only: it neither grants typing/row/continuation semantics nor
changes Core/CPS lowering, provider/frame behavior, or runtime admission/execution.  TASK-2013
remains in progress.
