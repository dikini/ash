# Surface Binding Scope Conformance Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make newline-separated surface statements lexically scoped by normatively lowering binding statements into `LET ... in cont`, then align parser, lowering, type checking, IR, interpreter, and conformance tests to that single rule.

**Architecture:** The phase is spec-first. First freeze the missing surface-to-core lowering contract in `docs/spec`, then implement that contract in the parser/lowering pipeline, then align type checking and runtime execution so all compile-time and runtime paths observe the same continuation-based scope. Close with conformance tests and examples that prove `ash check`, `ash run`, and `ash trace` now agree.

**Tech Stack:** Markdown spec/planning docs, Rust crates `ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`, integration tests, property tests with `proptest`.

---

### Task 1: Freeze the normative surface-to-core scoping rule

**Files:**
- Modify: `docs/spec/SPEC-002-SYNTAX.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `CHANGELOG.md`
- Create: `docs/plan/tasks/TASK-443-surface-statement-list-scoping-spec-amendment.md`

**Step 1: Write the failing documentation expectation**

Document the ambiguity explicitly in the task file and identify the missing normative rule: newline-separated statement lists currently have no single declared lowering into `LET` versus `SEQ`.

**Step 2: Verify the ambiguity exists in the current specs**

Run:

```bash
rg -n "LET|SEQ|statement list|block" docs/spec/SPEC-002-SYNTAX.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md
```

Expected: no one place normatively states the lexical block lowering rule for newline-separated surface statements.

**Step 3: Write the minimal spec amendments**

Add:

- one surface-to-core lowering rule for statement lists in `SPEC-002`
- one type-environment consequence note in `SPEC-003`
- one explicit cross-reference from `SPEC-004` and `SPEC-025` saying surface statement lists must already be lowered to canonical `LET`/`SEQ`

**Step 4: Verify the docs are coherent**

Run:

```bash
rg -n "lexical|statement list|lower" docs/spec/SPEC-002-SYNTAX.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md
```

Expected: one coherent lexical-scope story across all four specs.

**Step 5: Commit**

```bash
git add docs/spec/SPEC-002-SYNTAX.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md docs/plan/tasks/TASK-443-surface-statement-list-scoping-spec-amendment.md docs/plan/PLAN-INDEX.md CHANGELOG.md
git commit -m "docs(spec): freeze surface statement scoping contract"
```

### Task 2: Make parser and lowering produce the canonical lexical-block form

**Files:**
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Modify: `crates/ash-parser/tests/*`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-engine/tests/*`
- Create: `docs/plan/tasks/TASK-444-parser-and-lowering-lexical-block-normalization.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing parser/lowering tests**

Add focused tests proving that a surface block like:

```ash
let items = [1, 2, 3]
let first = items[0]
done
```

is normalized into nested `LET ... in cont` rather than sibling isolated statements.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p ash-parser lexical_block_scope -- --nocapture
cargo test -p ash-engine ordinary_file_lexical_block_scope -- --nocapture
```

Expected: current lowering/normalization does not yet guarantee the canonical lexical-block form.

**Step 3: Implement the minimal parser/lowering changes**

Update the surface parsing/lowering path so statement lists fold right-associatively:

- binding statements capture the lowered remainder as continuation
- non-binding statements lower via `SEQ stmt cont`

**Step 4: Run the focused tests to verify pass**

Run the same commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ash-parser/src/parse_workflow.rs crates/ash-parser/src/lib.rs crates/ash-parser/tests crates/ash-engine/src/lib.rs crates/ash-engine/tests docs/plan/tasks/TASK-444-parser-and-lowering-lexical-block-normalization.md CHANGELOG.md
git commit -m "feat(parser): normalize surface blocks into lexical continuations"
```

### Task 3: Align type checking with canonical lexical block scope

**Files:**
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/names.rs`
- Modify: `crates/ash-typeck/src/check_expr.rs`
- Modify: `crates/ash-typeck/tests/*`
- Create: `docs/plan/tasks/TASK-445-type-checker-lexical-scope-conformance.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing type-check tests**

Add tests covering:

- earlier `let` bindings visible in later statements of the same block
- unbound names rejected after normalization
- shadowing works only by later lexical binding in the same block

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p ash-typeck lexical_scope -- --nocapture
```

Expected: at least one case fails or depends on pre-normalized statement interpretation.

**Step 3: Implement the minimal type-checker changes**

Make type checking consume the canonical lowered shape consistently so the type environment extension mirrors the normative `LET ... in cont` rule.

**Step 4: Run the focused tests to verify pass**

Run the same command.

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ash-typeck/src/lib.rs crates/ash-typeck/src/names.rs crates/ash-typeck/src/check_expr.rs crates/ash-typeck/tests docs/plan/tasks/TASK-445-type-checker-lexical-scope-conformance.md CHANGELOG.md
git commit -m "fix(typeck): enforce lexical block scope for bindings"
```

### Task 4: Make interpreter execution faithful to the canonical lowered form

**Files:**
- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/eval.rs`
- Modify: `crates/ash-interp/tests/*`
- Modify: `crates/ash-engine/tests/*`
- Create: `docs/plan/tasks/TASK-446-interpreter-lexical-scope-and-seq-faithfulness.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing runtime/conformance tests**

Add regression tests proving that:

- `ash run` and `ash trace` can execute ordinary lexical blocks with later use of earlier `let` bindings
- true unbound names still fail at runtime with the expected rejection/error shape
- `SEQ` behavior remains consistent once the surface source is normalized

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p ash-interp lexical_scope -- --nocapture
cargo test -p ash-engine variables_example_scope -- --nocapture
```

Expected: current runtime behavior still disagrees with the canonical lowered shape in at least one focused case.

**Step 3: Implement the minimal runtime changes**

Update interpreter execution so the canonical lowered workflow carries bindings faithfully through continuation-owned scope and does not regress explicit `SEQ` semantics.

**Step 4: Run the focused tests to verify pass**

Run the same commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ash-interp/src/execute.rs crates/ash-interp/src/eval.rs crates/ash-interp/tests crates/ash-engine/tests docs/plan/tasks/TASK-446-interpreter-lexical-scope-and-seq-faithfulness.md CHANGELOG.md
git commit -m "fix(interp): honor lexical block bindings in execution"
```

### Task 5: Add end-to-end conformance coverage and phase closeout

**Files:**
- Modify: `examples/01-basics/02-variables.ash`
- Modify: `crates/ash-cli/tests/*`
- Modify: `crates/ash-engine/tests/*`
- Modify: `docs/plan/PLAN-INDEX.md`
- Create: `docs/plan/tasks/TASK-447-surface-binding-scope-conformance-closeout.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing end-to-end coverage**

Add integration coverage that runs the same lexical-scope examples through:

- `ash check`
- `ash run`
- `ash trace`

and asserts they agree on success for bound names and failure for truly unbound names.

**Step 2: Run the focused end-to-end tests to verify failure**

Run:

```bash
cargo test -p ash-cli variables_scope -- --nocapture
```

Expected: current end-to-end surfaces expose the disagreement.

**Step 3: Implement the minimal example/test closeout**

Refresh the example and integration assertions to codify the accepted lexical-scope contract.

**Step 4: Run verification**

Run:

```bash
cargo test --all
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
cargo doc --no-deps
```

Expected: all pass cleanly.

**Step 5: Commit**

```bash
git add examples/01-basics/02-variables.ash crates/ash-cli/tests crates/ash-engine/tests docs/plan/tasks/TASK-447-surface-binding-scope-conformance-closeout.md docs/plan/PLAN-INDEX.md CHANGELOG.md
git commit -m "test(cli): close lexical scope conformance phase"
```
