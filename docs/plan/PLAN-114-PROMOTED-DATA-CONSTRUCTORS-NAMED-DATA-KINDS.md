# PLAN-114: Promoted Data Constructors and Named Data Kinds

> **For Hermes:** This is an implementation plan for [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md). Use subagent-driven-development task-by-task only after the audit gate replaces downstream failing verification guards with exact focused commands. Do not implement beyond the owning SPEC.

**Goal:** Implement [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md) so the deferred DESIGN-034 gap has a concrete, tested, and non-overclaiming language/type-system slice.

**Architecture:** Promoted constructors are opt-in and distinct from sealed-domain markers and runtime constructors. Parser work is raw surface; shared identities/carriers live in `ash-core`; TypeEnv owns semantic checking; engine transports summaries without owning type semantics.

**Tech Stack:** Rust 2024, ash-parser, ash-core, ash-typeck, ash-engine, cargo fmt/check/clippy/test/doc.

---

**Status:** 📝 Planned
**Spec:** [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
**Design:** [DESIGN-036](../design/DESIGN-036-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md)
**Depends on:** SPEC-057 through SPEC-064 implemented MVPs
**Task range:** TASK-892 through TASK-897

## Task Breakdown

| Task | Description | Type | Est. Hours | Status |
|------|-------------|------|------------|--------|
| [TASK-892](tasks/TASK-892-promoted-constructor-audit-gate.md) | Audit live ADT, sealed-domain, type-function, normalizer, summary, and pattern seams before implementation | Docs/Substrate | 5 | 📝 Planned |
| [TASK-893](tasks/TASK-893-promoted-constructor-parser-surface.md) | Add chosen opt-in promoted-constructor/named-kind source surface and explicit unsupported-form diagnostics | Parser | 6 | 📝 Planned |
| [TASK-894](tasks/TASK-894-core-promoted-constructor-identities-and-summaries.md) | Add core promoted data-kind/constructor identities, type-level app carriers, and summary version contract | Core/Substrate | 8 | 📝 Planned |
| [TASK-895](tasks/TASK-895-typeenv-promoted-constructor-registration-and-kinding.md) | Register promoted identities in TypeEnv and validate kind/domain/source-ADT constraints | Typeck | 8 | 📝 Planned |
| [TASK-896](tasks/TASK-896-promoted-constructor-normalizer-proposition-and-non-interference.md) | Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference | Integration/Tests | 8 | 📝 Planned |
| [TASK-897](tasks/TASK-897-promoted-constructor-closeout.md) | Reconcile SPEC-065/PLAN-114 docs, acceptance matrix, broad gates, and review remediation | Docs/Closeout | 5 | 📝 Planned |

## Execution Tracks

- TASK-892: Audit live ADT, sealed-domain, type-function, normalizer, summary, and pattern seams before implementation
- TASK-893: Add chosen opt-in promoted-constructor/named-kind source surface and explicit unsupported-form diagnostics
- TASK-894: Add core promoted data-kind/constructor identities, type-level app carriers, and summary version contract
- TASK-895: Register promoted identities in TypeEnv and validate kind/domain/source-ADT constraints
- TASK-896: Integrate promoted apps with type functions/propositions and prove runtime ADT/sealed-domain non-interference
- TASK-897: Reconcile SPEC-065/PLAN-114 docs, acceptance matrix, broad gates, and review remediation

Total estimate: 40h before review remediation.

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
cargo doc --workspace --no-deps 2>&1 | tee /tmp/ash-plan-114-doc.log
! grep -i '^warning:' /tmp/ash-plan-114-doc.log
```

## Completion Checklist

- [ ] Audit gate artifact exists and downstream guards are patched.
- [ ] Parser/core/typeck/engine ownership matches [SPEC-065](../spec/SPEC-065-PROMOTED-DATA-CONSTRUCTORS-NAMED-DATA-KINDS.md).
- [ ] Acceptance/non-interference matrix maps every SPEC row to focused evidence.
- [ ] Broad workspace gates pass after the final code/doc change.
- [ ] Independent review remediation complete.
