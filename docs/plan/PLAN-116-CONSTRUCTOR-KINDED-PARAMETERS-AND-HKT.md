# PLAN-116: Constructor-Kinded Parameters and HKT

> **For Hermes:** This is an implementation plan for [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md). Use subagent-driven-development task-by-task only after the audit gate replaces downstream failing verification guards with exact focused commands. Do not implement beyond the owning SPEC.

**Goal:** Implement [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md) so the deferred DESIGN-034 gap has a concrete, tested, and non-overclaiming language/type-system slice.

**Architecture:** HKT is a cross-cutting type-system feature; do not implement as do-only magic. Parser work is raw surface; shared identities/carriers live in `ash-core`; TypeEnv owns semantic checking; engine transports summaries without owning type semantics.

**Tech Stack:** Rust 2024, ash-parser, ash-core, ash-typeck, ash-engine, cargo fmt/check/clippy/test/doc.

---

**Status:** ✅ Complete
**Spec:** [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Design:** [DESIGN-038](../design/DESIGN-038-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md)
**Depends on:** SPEC-057 through SPEC-064 implemented MVPs
**Task range:** TASK-904 through TASK-911

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-904](tasks/TASK-904-hkt-audit-gate.md) | Audit live interface/impl/type-param/do-target and generic-impl seams before HKT implementation | Docs/Substrate | 6 | ✅ Complete |
| [TASK-905](tasks/TASK-905-core-kinded-binder-and-constructor-var-carriers.md) | Add core kinded binder and constructor-variable application carriers | Core/Substrate | 8 | ✅ Complete |
| [TASK-906](tasks/TASK-906-parser-kinded-binder-surface.md) | Parse kinded binders in interfaces, impls, functions, type functions, and propositions at audited sites | Parser | 7 | ✅ Complete |
| [TASK-907](tasks/TASK-907-typeenv-constructor-variable-kinding-and-unification.md) | Track constructor variables, apply them by kind, and add non-inverting constructor unification | Typeck | 10 | ✅ Complete |
| [TASK-908](tasks/TASK-908-higher-kinded-interface-and-impl-coherence.md) | Register and resolve higher-kinded interface/impl evidence without overlap or output-directed selection | Typeck/Coherence | 10 | ✅ Complete |
| [TASK-909](tasks/TASK-909-monad-dictionary-do-target-resolution.md) | Route generalized do target resolution through `Monad<K>` evidence while preserving Act/Proc/Workflow bridge semantics | Typeck/Do | 8 | ✅ Complete |
| [TASK-910](tasks/TASK-910-hkt-diagnostics-and-acceptance-matrix.md) | Add Functor/Applicative/Monad diagnostics, acceptance, and non-interference matrix | Diagnostics/Tests | 7 | ✅ Complete |
| [TASK-911](tasks/TASK-911-hkt-closeout.md) | Reconcile SPEC-067/PLAN-116 docs, broad gates, and independent review remediation | Docs/Closeout | 5 | ✅ Complete |

## Execution Tracks

- TASK-904: Audit live interface/impl/type-param/do-target and generic-impl seams before HKT implementation
- TASK-905: Add core kinded binder and constructor-variable application carriers
- TASK-906: Parse kinded binders in interfaces, impls, functions, type functions, and propositions at audited sites
- TASK-907: Track constructor variables, apply them by kind, and add non-inverting constructor unification
- TASK-908: Register and resolve higher-kinded interface/impl evidence without overlap or output-directed selection
- TASK-909: Route generalized do target resolution through `Monad<K>` evidence while preserving Act/Proc/Workflow bridge semantics
- TASK-910: Add Functor/Applicative/Monad diagnostics, acceptance, and non-interference matrix
- TASK-911: Reconcile SPEC-067/PLAN-116 docs, broad gates, and independent review remediation

Total estimate: 61h before review remediation.

## Decision Gates

- D1: The audit gate is mandatory and must bind exact files, callsites, tests, and zero-test-safe commands before Rust changes.
- D2: Do not reuse ordinary `Type::Constructor` for computation-grade or partially applied type-level terms.
- D3: Do not broaden `do`, interface resolution, pattern checking, or summary transport outside this SPEC's explicit scope.
- D4: Every downstream task starts with a fail-closed verification guard until the audit gate patches it.
- D5: Closeout must update the spec index, PLAN-INDEX, task files, CHANGELOG, and acceptance/non-interference evidence.

## Verification Strategy

Each implementation task must run focused non-zero tests for its layer plus:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
```

Closeout additionally runs:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-116-doc.log
! grep -i '^warning:' /tmp/ash-plan-116-doc.log
```

## Completion Checklist

- [x] Audit gate artifact exists and downstream guards are patched.
- [x] Parser/core/typeck/engine ownership matches [SPEC-067](../spec/SPEC-067-CONSTRUCTOR-KINDED-PARAMETERS-AND-HKT.md).
- [x] Acceptance/non-interference matrix maps every SPEC row to focused evidence.
- [x] Broad workspace gate evidence recorded before final review remediation; later exact broad test reruns exposed only stale TASK-757/TASK-758 diagnostic expectations and one stale TASK-906 fail-closed interface-registration expectation now remediated.
- [x] Final independent Codex review completed; requested stale pending-review wording remediation and stale comprehension diagnostic-expectation remediation.

## Closeout Evidence

- TASK-904 through TASK-910 are complete with focused non-zero evidence recorded in their task files.
- TASK-910 records the SPEC-067 HKT acceptance and non-interference matrix for HKT-1 through HKT-8 in [docs/plan/audits/TASK-910-hkt-acceptance-matrix.md](audits/TASK-910-hkt-acceptance-matrix.md).
- TASK-911 records final status reconciliation, broad verification commands, the resolved sandbox test-gate blocker, and completed independent-review remediation in [docs/plan/audits/TASK-911-hkt-closeout.md](audits/TASK-911-hkt-closeout.md).
- The Phase 120 MVP preserves the SPEC-067 non-goals: higher-rank polymorphism, unrestricted source type lambdas, automatic do-target inference, law proving or automatic law assumption, arbitrary associated-type-family inversion during evidence search, broad multi-parameter constructor classes, and generalized runtime lowering through arbitrary user-defined Monad method bodies remain deferred.
- Earlier blocker cleared: the local mock-server port-binding failure in `ash-engine --test llm_engine_integration` was reproduced during closeout, then cleared by the orchestrator rerun `CODEX_NETWORK_ALLOW_LOCAL_BINDING=1 TMPDIR=/home/dikini/Projects/ash/.worktrees/phase-120-hkt/target/task911-tmp CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=target/task911-verify cargo test -p ash-engine --test llm_engine_integration -- --test-threads=1`, which passed 9 tests with 0 failures.
- Final independent Codex review completed and requested remediation for stale pending-review wording across PLAN-116/TASK-911/audit/CHANGELOG. Broad-test remediation updated stale TASK-757 and TASK-758 comprehension diagnostic expectations exposed by exact broad workspace test reruns to assert SPEC-067 missing `Monad<K>` evidence text instead of the old MVP dictionary wording, and updated a stale TASK-906 fail-closed interface-registration expectation now owned by TASK-908 higher-kinded interface registration.
- Post-remediation orchestrator verification completed: the final review findings were remediated, the focused stale-test reruns passed, and the final full `cargo test --workspace` passed against the remediated diff.
