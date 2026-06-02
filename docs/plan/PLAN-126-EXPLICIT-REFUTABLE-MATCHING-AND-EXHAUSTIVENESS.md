# PLAN-126: Explicit Refutable Matching and Exhaustiveness

> **For Hermes:** This is an implementation plan for [SPEC-076](../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md). Use subagent-driven-development task-by-task. Do not start Rust implementation until TASK-1001 replaces downstream fail-closed verification guards with exact focused commands.

**Goal:** Ban implicit refutable matching and enforce pattern exhaustiveness across Ash binder and eliminator constructs where closed coverage is available, require explicit blocked/deferred diagnostics where it is not, treat `if let ... else` as total by implicit complement, and preserve current selective `receive` as an explicit refutable filtering form.

**Architecture:** Parser remains raw-surface and span-preserving. `ash-typeck` owns type-aware irrefutability and exhaustive eliminator checks, consuming SPEC-068 pattern canonicalization. Runtime pattern failures become defensive fallbacks for unchecked IR rather than normal checked-source behavior.

**Tech Stack:** Rust 2024, ash-parser, ash-core, ash-typeck, ash-engine, ash-interp, ash-cli/LSP diagnostics, cargo fmt/check/clippy/test/doc.

---

**Status:** 📝 Planned
**Spec:** [SPEC-076](../spec/SPEC-076-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
**Design:** [DESIGN-044](../design/DESIGN-044-EXPLICIT-REFUTABLE-MATCHING-AND-EXHAUSTIVENESS.md)
**Depends on:** SPEC-068 implemented MVP
**Task range:** TASK-1000 through TASK-1008

## Task breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-1000](tasks/TASK-1000-explicit-refutable-matching-packet.md) | Create DESIGN-044/SPEC-076/PLAN-126 packet and register Phase 131 | Docs/Planning | 4 | ✅ Complete |
| [TASK-1001](tasks/TASK-1001-matching-semantics-audit-gate.md) | Audit all pattern-use callsites and replace fail-closed downstream verification guards | Docs/Substrate | 6 | ✅ Complete |
| [TASK-1002](tasks/TASK-1002-type-aware-irrefutable-pattern-api.md) | Add shared type-aware irrefutability API over canonical pattern types | Typeck/Substrate | 8 | ✅ Complete |
| [TASK-1003](tasks/TASK-1003-let-and-block-let-irrefutable-enforcement.md) | Enforce irrefutable patterns for pure block/core let binders | Typeck/Semantic | 8 | ✅ Complete |
| [TASK-1004](tasks/TASK-1004-workflow-and-operational-binder-irrefutable-enforcement.md) | Enforce irrefutable patterns for workflow/observe/spawn/split/loop binders | Typeck/Workflow | 10 | ✅ Complete |
| [TASK-1005](tasks/TASK-1005-deep-exhaustiveness-and-match-error-diagnostics.md) | Harden match exhaustiveness and missing-witness diagnostics | Typeck/Exhaustiveness | 10 | ✅ Complete |
| [TASK-1006](tasks/TASK-1006-with-error-total-handler-diagnostics.md) | Define and enforce or explicitly defer total `with_error` handler coverage | Typeck/Failure | 8 | ✅ Complete |
| [TASK-1007](tasks/TASK-1007-if-let-and-selective-receive-explicit-refutable-contract.md) | Refine `if let ... else` as total by implicit complement and preserve selective `receive` | Typeck/Semantic | 8 | ✅ Complete |
| [TASK-1008](tasks/TASK-1008-runtime-defensive-pattern-error-cleanup-closeout.md) | Verify runtime defensive error boundary, status surfaces, broad gates, and independent review | Runtime/Closeout | 8 | 📝 Planned |

Total estimate: 70h.

## Execution order

1. TASK-1001 maps every pattern-use callsite, current test surface, and expected RED failure.
2. TASK-1002 introduces the shared checker without wiring it broadly.
3. TASK-1003 and TASK-1004 wire binder positions.
4. TASK-1005 and TASK-1006 harden exhaustive eliminators.
5. TASK-1007 enforces the `if let ... else` implicit-complement contract while preserving selective receive filtering semantics.
6. TASK-1008 proves checked source no longer relies on runtime binder failure and closes the phase.

## Decision gates

- D1: The audit gate is mandatory and must bind exact files, callsites, tests, and non-zero focused commands before Rust implementation starts.
- D2: Binding positions must reject refutable patterns; they must not lower pattern failure into operational `fail`, `None`, `Err`, workflow rejection, or skipped execution.
- D3: Exhaustive eliminators must share the SPEC-068 constructor universe and blocked-canonicalization behavior.
- D4: `if let ... else` is a total two-branch eliminator over `P | not P`; `else` is mandatory, then/else result types must unify, irrefutable patterns are accepted with non-fatal unreachable-else diagnostics, impossible patterns are hard errors, and complement branches do not gain negative type refinement in this phase.
- D5: Current selective `receive` remains an explicit refutable filtering form in this phase; total protocol receive is deferred unless a later spec reclassifies it.
- D6: Runtime pattern errors such as expression `LetPatternBindFailed`, workflow `PatternMatchFailed`, and `NonExhaustiveMatch` remain defensive for unchecked IR/host values and must not be the ordinary outcome of checked source.
- D7: Diagnostics are part of the feature, not closeout polish: each semantic task must assert construct kind, type/witness, span, and likely rewrite where feasible.

## Verification strategy

Every implementation task must run its focused non-zero tests plus:

```bash
cargo fmt --check
git diff --check
cargo check --workspace
```

Closeout additionally runs:

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-126-doc.log
! grep -i '^warning:' /tmp/ash-plan-126-doc.log
```

TASK-1001 must replace all downstream fail-closed placeholder commands before TASK-1002 starts, and must freeze parser-entrypoint, yield-arm, surface-vs-core binder, and exact runtime-error variant evidence before implementation.

## Completion checklist

- [x] DESIGN-044, SPEC-076, PLAN-126, and TASK-1000 through TASK-1008 exist.
- [x] SPEC-076 is indexed in `docs/spec/README.md`.
- [x] Phase 131 is registered in `docs/plan/PLAN-INDEX.md`.
- [x] CHANGELOG records the docs packet.
- [x] TASK-1001 audit artifact exists and downstream verification guards are patched.
- [ ] Irrefutable binder enforcement is implemented and verified.
- [x] Exhaustive eliminator checks and diagnostics are implemented and verified.
- [x] Explicit refutable forms are preserved and documented.
- [ ] Runtime defensive pattern error boundary is verified.
