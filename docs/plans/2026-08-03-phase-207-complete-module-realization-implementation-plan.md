# Phase 207 Complete Module Realization Implementation Plan

> **For Codex:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Complete SPEC-103's one AST-driven file/inline module route through checked interfaces, Core/CPS, Engine admission, and matching CLI/daemon terminals.

**Architecture:** Preserve the existing non-authoritative TASK-2057--2062/2066 handoffs and make each missing semantic boundary explicit. TASK-2063 will convert a fully validated, canonical module Core/CPS closure into an Engine-private admission request; only that request may reach the checked-CPS executor. TASK-2064 then completes remaining source/binder/interface integration and proves file/inline plus client parity; TASK-2065 closes the phase only from executed evidence and reconciled references.

**Tech Stack:** Rust 2024; `ash-parser`, `ash-core`, `ash-typeck`, `ash-engine`, `ash-cli`, daemon client; `proptest`; repository documentation validators.

---

## Preconditions and non-negotiable contract

- Authoritative target: `docs/spec/SPEC-103-MODULE-REALIZATION-AND-OPERATIONAL-SEMANTICS.md` §§5--11.
- The task records `TASK-2063`, `TASK-2064`, and `TASK-2065` are the work authorizations. Promote a planned task to **In progress** and add its semantic-record/coverage state before its Rust implementation begins.
- The existing TASK-2057--2062 and TASK-2066 files are an uncommitted partial baseline. Preserve and verify their behavior; do not recast their bounded carriers as full interface, binder, or runtime authority.
- No raw source scan, loader-private export table, parser graph, legacy graph, or direct evaluator may authorize module graph facts, imports, lowering, admission, or execution.
- A task can report `implemented` only if the full clause it owns is realized; tests are `tested`, not `proved`.

## Task 1: TASK-2063 -- Engine-sealed linked-module admission

**Files:**

- Create: `crates/ash-engine/tests/task_2063_engine_linked_module_admission.rs`
- Create: `crates/ash-engine/src/linked_module_admission.rs`
- Modify: `crates/ash-engine/src/lib.rs`
- Modify: `crates/ash-core/src/module_lowering.rs`
- Modify: `crates/ash-typeck/src/module_core_cps_lowering.rs`
- Modify: `docs/plan/tasks/TASK-2063-engine-linked-module-admission.md`
- Modify: `docs/plan/PLAN-207-COMPLETE-MODULE-REALIZATION.md`, `docs/plan/PLAN-INDEX.md`, `docs/plan/SEMANTIC-RULE-COVERAGE.md`, `docs/plan/semantic-task-records.json`, `docs/spec/SEMANTIC-TRACEABILITY.json`, `docs/plan/audits/AUDIT-207-module-realization-seams.md`, `docs/reference/language/lexical-and-modules/modules-imports-and-visibility.md`, and `CHANGELOG.md`

**Step 1: Add the failing Engine admission tests.**

Build public TASK-2062 fixture pairs for a root and child module, with a valid handler-free CPS root and canonical artifact/interface dependency metadata. Test that Engine linking:

1. accepts the complete closure and yields an opaque, Engine-issued linked request;
2. rejects an absent dependency, duplicate/mismatched module key, Core/CPS artifact mismatch, unsupported artifact/interface schema, origin mismatch, stale imported defining identity, and a failed/incomplete dependency before execution;
3. rejects an attempted raw source, legacy loader, or direct-evaluator substitute before any runtime route is selected; and
4. executes only an Engine-issued linked request through the checked-CPS dispatcher and returns the expected normalized terminal.

Run: `cargo test -p ash-engine --test task_2063_engine_linked_module_admission`

Expected: compile failure because the linked-module input/admission API does not yet exist.

**Step 2: Extend the TASK-2062 transport carrier with immutable checked-interface continuity facts.**

`ModuleCoreArtifact` and `ModuleCpsArtifact` must retain the exact `PublicModuleInterface` snapshot that the finalizer supplied; construction must reject/avoid interface-artifact divergence. `lower_finalized_module_to_core_cps` must derive that snapshot from the finalizer wrapper. The public data remains forgeable transport data and cannot become an admission credential.

Run focused predecessor tests:

```text
cargo test -p ash-core module_lowering
cargo test -p ash-typeck --test task_2062_module_core_cps_lowering
```

Expected: existing tests remain green; add a regression that validates exact interface/artifact propagation.

**Step 3: Implement deterministic closure validation and Engine sealing.**

In `linked_module_admission.rs`, add a public input-only closure container and error type plus crate-private sealed request/admission types. Validate, in deterministic `ModuleKey` order, all of the following before a sealed value exists:

- exactly one root with a canonical entry identity;
- one matching Core/CPS/interface artifact triplet per key;
- supported module-artifact and public-interface schema versions;
- exact key, origin, structural-parent/child, and imported defining-identity continuity between Core and CPS transport data;
- every declared checked-interface dependency and reachable import identity resolves to a present, validated closure member;
- no duplicate, cycle, failed, incomplete, stale, or forged entry.

After validation only, terminalize the root CPS artifact under the Engine-owned answer continuation and create a `CheckedCpsEntryAdmission` inside `ash-engine`. Do not expose its CPS term or make its constructor public. Add an Engine method that accepts the sealed linked artifact, manufactures the existing Engine request/control envelope, and dispatches only through the checked-CPS execution path.

**Step 4: Verify RED becomes GREEN.**

Run:

```text
cargo test -p ash-engine --test task_2063_engine_linked_module_admission
cargo test -p ash-core module_lowering
cargo test -p ash-typeck --test task_2062_module_core_cps_lowering
cargo fmt --check
cargo clippy -p ash-engine --all-targets --all-features -- -D warnings
```

Expected: all pass, and every negative/mutation assertion fails before execution rather than taking a fallback route.

**Step 5: Record exactly earned Task 2063 evidence.**

Update the task, Phase 207 plan/index, `MOD-REAL-006` coverage row, semantic task record, traceability, seam audit, implementation-backed language reference, and changelog. Mark only TASK-2063's concrete closure/admission boundary `implemented / tested / below_spec`; do not claim real-source file/inline or client parity.

## Task 2: Promote and implement TASK-2064 source-to-client conformance

**Files:**

- Create focused parser/core/typeck/engine/CLI/daemon rule-indexed tests and a shared module conformance fixture builder.
- Modify only the parser, graph, finalizer/binder, lowering, Engine, and clients shown necessary by a failing `MOD-REAL-*` rule fixture.
- Modify the TASK-2064 task record, coverage/semantic records/traceability, Phase 207 plan/index, seam audit, language reference, orientation indexes if a new reference/note/spec is added, and changelog.

**Step 1: Promote the task and add RED corpus fixtures.**

Set TASK-2064 to **In progress** in all required tracking records. Add one compact positive and one negative fixture per `MOD-REAL-001`--`006`, then a property generator that materializes equivalent file-backed and inline declaration trees. Ensure each fixture is first failing for a specific missing target clause, not simply unsupported legacy behavior.

**Step 2: Complete the parser/binder/interface route one rule at a time.**

For each failing test, make the smallest semantic implementation that passes it:

1. traverse only expanded parsed `ModuleFile`/`ModuleUnit` nodes for structural and import edges, with anchored missing/duplicate/structural-cycle errors;
2. collect a complete checked private/public interface, including typed namespaces and explicit aliases/re-exports, atomically; do not flatten child exports implicitly;
3. resolve parsed `use` and every existing visibility spelling through finalized interfaces, reject ambiguity/inaccessibility/import cycles before registration, and preserve defining identities;
4. lower checked module definitions and resolved bindings through Core/CPS without source rediscovery, then form the validated Engine closure from Task 1.

Run focused tests after each Green transition. Add a property or mutation test for order independence, alias identity, no implicit flattening, source-lookalike resistance, and fallback resistance. Reject mutation attempts at the intended boundary rather than adding test-only bypasses.

**Step 3: Establish true source and client parity.**

Use one identical admitted multi-module `fn main` source tree for both file and inline forms. Compare normalized interface, Core, CPS, closure/admission result, and then normalized terminal output through CLI and daemon under identical inputs, bindings, and run control. The daemon must be a client of the same Engine request path; it may not parse/evaluate independently.

Run:

```text
cargo test -p ash-parser module
cargo test -p ash-core module
cargo test -p ash-typeck module
cargo test -p ash-engine module
cargo test -p ash-cli module
```

**Step 4: Record independently earned rule evidence.**

Update every `MOD-REAL-*` coverage result and traceability mapping from actual focused command results. Do not mark a rule `matches_spec` until source parity, atomic failure behavior, one Engine route, and terminal parity evidence covers all of its clauses.

## Task 3: TASK-2065 closeout and reference reconciliation

**Files:**

- Create: a closeout checker/test that validates Phase 207 coverage axes, task handoffs, scanner classification, and reference claims.
- Modify: `CHANGELOG.md`, PLAN-207, PLAN-INDEX, semantic coverage, semantic task records, semantic traceability, AUDIT-207, language reference, `docs/spec/SPEC-INDEX.md` and/or `docs/notes/NOTE-INDEX.md` only if their indexed contents changed.

**Step 1: Add the failing closeout checker.**

It must fail when any SPEC-103 conformance clause has no implementation/evidence/parity result, a planned module task has no record, a live scanner is unclassified or semantic-authorizing, a direct evaluator is reachable from the linked route, or a reference overclaims its recorded evidence.

**Step 2: Run independent reviews and remediate.**

Dispatch a spec-compliance review followed by a Rust/code-quality review across parser, core, typeck, Engine, clients, and documentation. Resolve every blocking finding or retain it as an explicit `partial / below_spec` target gap; a gap means Phase 207 remains open.

**Step 3: Run the full gate and reconcile references.**

```text
cargo fmt --check
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
python3 tools/docs/validate_orientation_indexes.py --self-test
bash scripts/check-docs-gate.sh
git diff --check
```

Only after the closeout checker and every command passes, mark task/plan status and `MOD-REAL-*` axes from the recorded evidence. Add Common Changelog entries describing the realized module route and any reference corrections.
