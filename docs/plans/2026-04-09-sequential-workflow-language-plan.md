# Sequential Workflow Language Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove `par` from the Ash language, leaving a single workflow as a sequential process and relocating concurrency examples to communicating workflows/processes.

**Architecture:** This phase is spec-first and task-driven. First freeze the language change in a dedicated task file and amend the normative specs so `Par` is no longer part of the active language contract. Then remove the feature from parser, lowering, core AST, typing, runtime, and user-facing surfaces, and finally update conformance/reference/examples so the active corpus reflects the sequential-only workflow model while preserving historical records.

**Tech Stack:** Markdown planning/spec docs, Rust crates `ash-core`, `ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`, `ash-repl`, integration tests, workflow fixtures, examples.

---

### Task 1: Freeze the task and spec contract for sequential-only workflows

**Files:**
- Create: `docs/plan/tasks/TASK-448-remove-par-form-and-make-single-workflows-sequential.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/spec/SPEC-001-IR.md`
- Modify: `docs/spec/SPEC-002-SURFACE.md`
- Modify: `docs/spec/SPEC-003-TYPE-SYSTEM.md`
- Modify: `docs/spec/SPEC-004-SEMANTICS.md`
- Modify: `docs/spec/SPEC-022-WORKFLOW-TYPING.md`
- Modify: `docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md`
- Modify: `docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the task file**

Document the required scope: `par` is removed from the current language contract, historical records remain, and single-workflow sequencing becomes the active spec truth.

**Step 2: Verify the current specs still treat `Par` as normative**

Run:

```bash
rg -n "\\bPar\\b|\\bpar\\b|parallel" docs/spec/SPEC-001-IR.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-022-WORKFLOW-TYPING.md docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md
```

Expected: active normative mentions of `Par` and workflow-internal concurrency are present.

**Step 3: Write the minimal spec amendments**

Remove `Par` from workflow inventories and rules, narrow nondeterminism language to the remaining helper/runtime-owned surfaces, and state explicitly that a single workflow is sequential while concurrent systems are modeled by communicating workflows.

**Step 4: Verify the spec corpus is coherent**

Run:

```bash
rg -n "\\bPar\\b|\\bpar\\b" docs/spec
```

Expected: no remaining normative `Par` contract in current active specs, or only explicitly historical references that are intentionally retained outside the language definition.

**Step 5: Commit**

```bash
git add docs/plan/tasks/TASK-448-remove-par-form-and-make-single-workflows-sequential.md docs/plan/PLAN-INDEX.md docs/spec/SPEC-001-IR.md docs/spec/SPEC-002-SURFACE.md docs/spec/SPEC-003-TYPE-SYSTEM.md docs/spec/SPEC-004-SEMANTICS.md docs/spec/SPEC-022-WORKFLOW-TYPING.md docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md CHANGELOG.md
git commit -m "docs(spec): remove par from active language contract"
```

### Task 2: Remove `par` from parsing and lowering

**Files:**
- Modify: `crates/ash-parser/src/token.rs`
- Modify: `crates/ash-parser/src/lexer.rs`
- Modify: `crates/ash-parser/src/parse_pattern.rs`
- Modify: `crates/ash-parser/src/parse_expr.rs`
- Modify: `crates/ash-parser/src/parse_policy.rs`
- Modify: `crates/ash-parser/src/parse_workflow.rs`
- Modify: `crates/ash-parser/src/surface.rs`
- Modify: `crates/ash-parser/src/desugar.rs`
- Modify: `crates/ash-parser/src/lower.rs`
- Modify: `crates/ash-parser/src/error_recovery.rs`
- Modify: `crates/ash-parser/tests/lexer_props.rs`
- Modify: `crates/ash-engine/src/parse.rs`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing parser/lowering tests**

Add focused tests proving:

- `par { ... }` no longer parses;
- the parser keyword inventory no longer reserves `par`;
- lowering no longer contains a `SurfaceWorkflow::Par -> CoreWorkflow::Par` path.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p ash-parser par -- --nocapture
cargo test -p ash-engine parse -- --nocapture
```

Expected: current parser/lowering still accepts `par`.

**Step 3: Implement the minimal parser/lowering removal**

Delete the token, parser production, surface AST variant, desugar path, and lowering path for `par`, and update recovery/keyword tables accordingly.

**Step 4: Run the focused tests to verify pass**

Run the same commands.

Expected: PASS, with `par` rejected at the parser boundary.

**Step 5: Commit**

```bash
git add crates/ash-parser/src/token.rs crates/ash-parser/src/lexer.rs crates/ash-parser/src/parse_pattern.rs crates/ash-parser/src/parse_expr.rs crates/ash-parser/src/parse_policy.rs crates/ash-parser/src/parse_workflow.rs crates/ash-parser/src/surface.rs crates/ash-parser/src/desugar.rs crates/ash-parser/src/lower.rs crates/ash-parser/src/error_recovery.rs crates/ash-parser/tests/lexer_props.rs crates/ash-engine/src/parse.rs CHANGELOG.md
git commit -m "feat(parser): remove par workflow form"
```

### Task 3: Remove `Par` from core AST, typing, and runtime execution

**Files:**
- Modify: `crates/ash-core/src/ast.rs`
- Modify: `crates/ash-core/src/visualize.rs`
- Modify: `crates/ash-core/src/workflow_contract.rs`
- Modify: `crates/ash-typeck/src/lib.rs`
- Modify: `crates/ash-typeck/src/effect.rs`
- Modify: `crates/ash-typeck/src/names.rs`
- Modify: `crates/ash-typeck/src/obligations.rs`
- Modify: `crates/ash-typeck/src/capability_check.rs`
- Modify: `crates/ash-interp/src/execute.rs`
- Modify: `crates/ash-interp/src/error.rs`
- Modify: `crates/ash-interp/src/execution_record.rs`
- Modify: `crates/ash-interp/src/runtime_outcome_state.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing Rust-side tests**

Add or update tests proving:

- no AST/public workflow contract includes `Par`;
- type/effect analysis no longer matches on `Par`;
- interpreter execution has no `Workflow::Par` branch;
- any runtime-only parallel aggregation helpers left behind are removed or narrowed to non-language concurrency primitives.

**Step 2: Run the focused tests to verify failure**

Run:

```bash
cargo test -p ash-core par -- --nocapture
cargo test -p ash-typeck par -- --nocapture
cargo test -p ash-interp par -- --nocapture
```

Expected: current code still exposes `Par` and related helpers.

**Step 3: Implement the minimal core/type/runtime removal**

Delete the `Par` variants and match arms, remove unused parallel-aggregation helpers that only exist for the language form, and keep unrelated runtime concurrency support intact.

**Step 4: Run the focused tests to verify pass**

Run the same commands.

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ash-core/src/ast.rs crates/ash-core/src/visualize.rs crates/ash-core/src/workflow_contract.rs crates/ash-typeck/src/lib.rs crates/ash-typeck/src/effect.rs crates/ash-typeck/src/names.rs crates/ash-typeck/src/obligations.rs crates/ash-typeck/src/capability_check.rs crates/ash-interp/src/execute.rs crates/ash-interp/src/error.rs crates/ash-interp/src/execution_record.rs crates/ash-interp/src/runtime_outcome_state.rs crates/ash-engine/src/lib.rs CHANGELOG.md
git commit -m "refactor(core): remove Par from workflow execution model"
```

### Task 4: Replace `par` fixtures, examples, and user-facing surfaces

**Files:**
- Modify: `examples/README.md`
- Modify: `examples/02-control-flow/README.md`
- Modify: `examples/02-control-flow/03-parallel.ash`
- Modify: `examples/code_review.ash`
- Modify: `examples/multi_agent_research.ash`
- Modify: `examples/simple_workflow.ash`
- Modify: `examples/04-real-world/customer-support.ash`
- Modify: `tests/workflows/code_review.ash`
- Modify: `tests/workflows/multi_agent_research.ash`
- Modify: `docs/TUTORIAL.md`
- Modify: `docs/spec/SPEC-023-PROXY-WORKFLOWS.md`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing example/integration expectations**

Identify each shipped example or fixture that still demonstrates `par`, and add/update tests or fixture assertions so current examples reflect sequential or communicating-workflow modeling instead.

**Step 2: Run focused checks to verify stale examples remain**

Run:

```bash
rg -n "\\bpar\\b|Parallel" examples tests/workflows docs/TUTORIAL.md docs/spec/SPEC-023-PROXY-WORKFLOWS.md
```

Expected: multiple active user-facing surfaces still demonstrate `par`.

**Step 3: Implement the minimal example migration**

Rename, rewrite, or replace the affected examples so they either demonstrate sequential workflows or message-passing/process composition rather than workflow-internal parallel branches.

**Step 4: Re-run the focused checks**

Run the same command.

Expected: no active example/tutorial/workflow fixture still teaches `par`.

**Step 5: Commit**

```bash
git add examples/README.md examples/02-control-flow/README.md examples/02-control-flow/03-parallel.ash examples/code_review.ash examples/multi_agent_research.ash examples/simple_workflow.ash examples/04-real-world/customer-support.ash tests/workflows/code_review.ash tests/workflows/multi_agent_research.ash docs/TUTORIAL.md docs/spec/SPEC-023-PROXY-WORKFLOWS.md CHANGELOG.md
git commit -m "docs(examples): replace par-based workflow examples"
```

### Task 5: Remove active conformance/reference dependence on `Par` and verify the workspace

**Files:**
- Modify: `docs/reference/canonical-ir-semantics-corpus.md`
- Modify: `docs/reference/canonical-semantics-result-format.md`
- Modify: `docs/reference/formalization-boundary.md`
- Modify: `docs/reference/semantic-execution-record-contract.md`
- Modify: `docs/reference/type-to-runtime-contract.md`
- Modify: `crates/ash-repl/src/ast.rs`
- Modify: `CHANGELOG.md`

**Step 1: Write the failing reference/conformance expectation**

Document and verify that active conformance/reference material still assumes `Par` is a live current-language feature.

**Step 2: Run focused checks to verify failure**

Run:

```bash
rg -n "\\bPar\\b|\\bpar\\b" docs/reference crates/ash-repl/src/ast.rs
```

Expected: active reference material still encodes `Par` cases or display support.

**Step 3: Implement the minimal reference cleanup**

Remove active-language `Par` dependence from conformance/reference material while keeping historical planning/task records untouched. Where necessary, narrow nondeterminism language to the remaining helper/runtime-owned sources.

**Step 4: Run full verification**

Run:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
cargo doc --no-deps
```

Expected: all pass cleanly.

**Step 5: Commit**

```bash
git add docs/reference/canonical-ir-semantics-corpus.md docs/reference/canonical-semantics-result-format.md docs/reference/formalization-boundary.md docs/reference/semantic-execution-record-contract.md docs/reference/type-to-runtime-contract.md crates/ash-repl/src/ast.rs CHANGELOG.md
git commit -m "test(conformance): remove active Par corpus dependence"
```
