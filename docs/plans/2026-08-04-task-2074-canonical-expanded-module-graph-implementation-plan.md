# TASK-2074 Canonical Expanded Module Graph Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build one atomic, AST-only expanded graph over every canonical parsed module key.

**Architecture:** `ash-parser` consumes and owns `CanonicalModuleGraph`, performs a syntax-summary
dependency prepass, shallowly expands each keyed `ModuleBody`, and publishes a read-only exact-key
map. It never calls Engine loading, scans source text, or performs filesystem/path lookup.

**Tech Stack:** Rust 2024, `ash-parser`, `ash-core::ModuleKey`, existing surface expansion and
`proptest` infrastructure.

---

### Task 1: Activate TASK-2074

**Files:**
- Modify: `docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`

**Step 1:** Change exact status from `Planned` to `In progress`.

**Step 2:** Add the task-owned coverage section, active record, deferred implementation/test nodes,
and focused verification command. Do not mark evidence tested.

**Step 3:** Run `python3 tools/docs/validate_semantic_task_records.py --self-test` and expect PASS.

**Step 4:** Commit the activation metadata with `docs: activate TASK-2074 expanded graph`.

### Task 2: Establish RED shallow-expansion and retention tests

**Files:**
- Create: `crates/ash-parser/tests/task_2074_canonical_expanded_module_graph.rs`
- Modify: `crates/ash-parser/Cargo.toml` only if the existing test configuration requires it

**Step 1:** Add a test constructing one canonical graph whose body interleaves use, macro-expanded
definition, and child declaration; assert the future expanded record preserves use and item order
while expanding only the direct definition.

**Step 2:** Add a nested-inline case asserting child-generated origin/hygiene sidecars occur only
under the child key.

**Step 3:** Run `cargo test -p ash-parser --test task_2074_canonical_expanded_module_graph` and
expect compile failure because the public carrier/API does not exist.

**Step 4:** Commit RED tests with `test(parser): specify canonical expanded graph`.

### Task 3: Add the shallow `ModuleBody` expansion seam

**Files:**
- Modify: `crates/ash-parser/src/module.rs`
- Modify: `crates/ash-parser/src/surface.rs`

**Step 1:** Add a crate-private function taking one `ModuleBody` and syntax environment and returning
an expanded body plus module-local diagnostics/origin/hygiene sidecars.

**Step 2:** Iterate `ModuleItem` in source order; expand `Definition` items, clone `Use` and
`ModuleDecl` items unchanged, and reconstruct with `ModuleBody::from_items`.

**Step 3:** Run the focused target and expect the carrier/API failures to remain while the new
internal seam compiles.

**Step 4:** Commit with `feat(parser): add shallow module-body expansion`.

### Task 4: Specify and implement the syntax-only prepass

**Files:**
- Modify: `crates/ash-parser/tests/task_2074_canonical_expanded_module_graph.rs`
- Create: `crates/ash-parser/src/canonical_syntax_dependencies.rs`
- Modify: `crates/ash-parser/src/lib.rs`

**Step 1:** Add RED cases for local/public macro and notation summaries, canonical alias resolution,
private/non-syntax/missing summary rejection, provider-before-consumer order, and an anchored two-
and three-module syntax cycle.

**Step 2:** Add parser-private summary/edge types keyed by `ModuleKey` and declaration/use spans.

**Step 3:** Collect summaries only from graph AST, resolve only admitted syntax imports, perform a
stable topological sort, and return no value on any error.

**Step 4:** Assert imported notation remains inactive without a canonical summary and item-generating
macro attempts reject.

**Step 5:** Run the focused target; expect only expanded-carrier cases to remain RED.

**Step 6:** Commit with `feat(parser): add canonical syntax dependency prepass`.

### Task 5: Implement the atomic expanded graph carrier

**Files:**
- Create: `crates/ash-parser/src/canonical_expanded_module_graph.rs`
- Modify: `crates/ash-parser/src/lib.rs`
- Modify: `crates/ash-parser/tests/task_2074_canonical_expanded_module_graph.rs`

**Step 1:** Define private fields for the owned parsed graph and exact keyed expanded records; expose
only `try_expand`, `parsed_graph`, `module`, and ordered `modules` read APIs.

**Step 2:** Stage every result in a private map following prepass order. Wrap failures with module
identity, origin/path, and exact source anchor.

**Step 3:** Compare staged and parsed key sets exactly before constructing the public carrier.

**Step 4:** Run the focused target and expect the initial positive/negative cases to pass.

**Step 5:** Commit with `feat(parser): add canonical expanded module graph`.

### Task 6: Add mutation, parity, property, and authority fences

**Files:**
- Modify: `crates/ash-parser/tests/task_2074_canonical_expanded_module_graph.rs`

**Step 1:** Add mutations for missing/extra/wrong key, source-order loss, moved inline sidecar,
altered syntax edge, late expansion failure, and syntax cycle.

**Step 2:** Add normalized equivalent file/inline bodies and a generated 16+ case property varying
module depth, source form, uses, and direct macro expansions.

**Step 3:** Add a no-FS/authority fence proving the module has no dependency on `ash-engine`,
`ModuleLoader`, source reads, path caches, Core/CPS, or runtime carriers.

**Step 4:** Run the focused target and expect PASS.

**Step 5:** Commit with `test(parser): cover expanded graph invariants`.

### Task 7: Regression, review, and closeout

**Files:**
- Modify: `docs/plan/tasks/TASK-2074-canonical-expanded-module-graph.md`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/SEMANTIC-RULE-COVERAGE.md`
- Modify: `docs/plan/audits/AUDIT-207-module-realization-seams.md`
- Modify: `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`
- Modify: `docs/plan/semantic-task-records.json`
- Modify: `docs/spec/SEMANTIC-TRACEABILITY.json`
- Modify: `CHANGELOG.md`

**Step 1:** Run the focused target plus existing parser expansion and TASK-2067 graph targets.

**Step 2:** Run `cargo fmt --check`, `cargo clippy -p ash-parser --all-targets --all-features -- -D warnings`,
and `cargo test -p ash-parser`; expect PASS.

**Step 3:** Request spec review, code-quality review, and independent QA; fix every blocking issue.

**Step 4:** Promote only observed implementation/test evidence and retain `partial / tested /
below_spec` for the target rule.

**Step 5:** Run semantic-record, traceability, orientation, docs-gate, and diff checks.

**Step 6:** Commit with `feat(parser): complete canonical expanded module graph`.
