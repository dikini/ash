# PLAN-115: Type Holes and Partial Type-Constructor Application

> **For Hermes:** This is an implementation plan for [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md). Use subagent-driven-development task-by-task only after the audit gate replaces downstream failing verification guards with exact focused commands. Do not implement beyond the owning SPEC.

**Goal:** Implement [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md) so the deferred DESIGN-034 gap has a concrete, tested, and non-overclaiming language/type-system slice.

**Architecture:** Explicit `_` holes are not implicit currying and do not solve by inversion. Parser work is raw surface; shared identities/carriers live in `ash-core`; TypeEnv owns semantic checking; engine transports summaries without owning type semantics.

**Tech Stack:** Rust 2024, ash-parser, ash-core, ash-typeck, ash-engine, cargo fmt/check/clippy/test/doc.

---

**Status:** ✅ Complete
**Spec:** [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md)
**Design:** [DESIGN-037](../design/DESIGN-037-TYPE-HOLES-PARTIAL-TYPE-CONSTRUCTOR-APPLICATION.md)
**Depends on:** SPEC-057 through SPEC-064 implemented MVPs
**Task range:** TASK-898 through TASK-903

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-898](tasks/TASK-898-type-hole-audit-gate.md) | Audit parser/core/typeck/do-target/type-function wildcard seams and freeze enabled hole positions | Docs/Substrate | 5 | ✅ Complete |
| [TASK-899](tasks/TASK-899-core-type-hole-and-partial-application-carriers.md) | Add core hole identity, partial-argument, and partial-constructor carriers without nominal saturation | Core/Substrate | 7 | ✅ Complete |
| [TASK-900](tasks/TASK-900-parser-type-hole-surface.md) | Parse `_` holes at audited type-expression positions and preserve spans distinctly from type-function pattern wildcards | Parser | 6 | ✅ Complete |
| [TASK-901](tasks/TASK-901-typeenv-partial-constructor-kinding.md) | Elaborate holes and partial applications with kind/arity/ambiguity validation | Typeck | 8 | ✅ Complete |
| [TASK-902](tasks/TASK-902-do-target-partial-application-integration.md) | Allow do-target shape elaboration for unary partial targets such as `Result<_, E>` while preserving missing-Monad evidence boundaries | Typeck/Do | 6 | ✅ Complete |
| [TASK-903](tasks/TASK-903-type-hole-closeout.md) | Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation | Docs/Closeout | 5 | ✅ Complete |

## Execution Tracks

- TASK-898: Audit parser/core/typeck/do-target/type-function wildcard seams and freeze enabled hole positions
- TASK-899: Add core hole identity, partial-argument, and partial-constructor carriers without nominal saturation
- TASK-900: Parse `_` holes at audited type-expression positions and preserve spans distinctly from type-function pattern wildcards
- TASK-901: Elaborate holes and partial applications with kind/arity/ambiguity validation
- TASK-902: Allow do-target shape elaboration for unary partial targets such as `Result<_, E>` while preserving missing-Monad evidence boundaries
- TASK-903: Reconcile SPEC-066/PLAN-115 docs, diagnostics, acceptance matrix, broad gates, and review remediation

Total estimate: 37h before review remediation.

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
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-115-doc.log
! grep -i '^warning:' /tmp/ash-plan-115-doc.log
```

## Completion Checklist

- [x] Audit gate artifact exists and downstream guards are patched.
- [x] Parser/core/typeck/engine ownership matches [SPEC-066](../spec/SPEC-066-TYPE-HOLES-PARTIAL-CONSTRUCTOR-APPLICATION.md).
- [x] Acceptance/non-interference matrix maps every SPEC row to focused evidence.
- [x] Broad workspace gates pass after the final code/doc change.
- [x] Independent review remediation complete.

## Closeout Evidence

- Acceptance matrix: [TASK-903 type-hole acceptance matrix](audits/TASK-903-type-hole-acceptance-matrix.md) maps SPEC-066 H-1 through H-6 to focused non-zero tests.
- Focused tests: TASK-899 core carriers (3 tests), TASK-900 parser surface (4 tests), TASK-901 TypeEnv kinding (9 tests), and TASK-902 do-target integration (6 tests) pass.
- Review remediation: stale do-target explicit-argument assertions were updated and `cargo test -p ash-typeck --lib do_target -- --test-threads=1` passes with 9 tests.
- Broad gates: closeout ran fmt, diff whitespace, workspace check, clippy, test, and doc warning gates after final code/doc changes.
