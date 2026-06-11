# TASK-1394: Reference, generated-test handoff, and closeout

## Status: ✅ Complete

## Description

Reconcile reference docs, generated-test handoff docs, changelog, and broad verification for the new law/proof surface.

## Specification Reference

- [PLAN-138: Stdlib Algebra Laws and Pure-Carrier Proofs](../PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md)
- [SPEC-078: Standard Algebra Library and Monad Remediation](../../spec/SPEC-078-STANDARD-ALGEBRA-LIBRARY-AND-MONAD-REMEDIATION.md)
- [DESIGN-NOTE-INTERFACE-LAWS](../../design/DESIGN-NOTE-INTERFACE-LAWS.md)

## Dependencies

- TASK-1388 through TASK-1393 completed.

## Deferral / Planned-Feature Reconciliation

Generated law-test execution remains owned by SPEC-077/TASK-1029 or successor tasks unless implemented in this phase. Law declarations and proof declarations are distinct surfaces.

## Requirements

### Functional Requirements

- Update reference docs to say laws are source-visible in `std/src/algebra`, not only handoff prose.
- Separate law declaration status from proof status in documentation.
- Add a changelog entry under `[Unreleased]`.
- Run docs and code gates.
- Run independent review focused on: law names matching interface methods, no fake proof success, `Result` domain failure distinct from operational bottom, Applicative composition not silently omitted, generated-test handoff consistency.

## Files

- Modify: `reference/stdlib/algebra.md`
- Modify: `CHANGELOG.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Modify: `docs/plan/PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md`
- Inspect: `docs/plan/audits/TASK-1026-algebra-law-test-handoff.md`

## TDD Steps

### Step 1: Update reference docs

Refresh `reference/stdlib/algebra.md` to document source-visible law declarations.

### Step 2: Update changelog

Add Phase 138 closeout entry under `[Unreleased]`.

### Step 3: Update PLAN-INDEX

Mark Phase 138 task rows as Complete in both summary and phase-body tables.

### Step 4: Run full gates

```bash
bash scripts/check-rust-format.sh
git diff --check
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo check --workspace
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-rust-clippy.sh
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-doc-tests.sh
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true bash scripts/check-rust-tests.sh --workspace --all-targets
RUSTC_WRAPPER= CARGO_NET_OFFLINE=true cargo doc --workspace --no-deps 2>&1 | tee /tmp/phase-138-main-doc.log
! grep -i '^warning:' /tmp/phase-138-main-doc.log
```

### Step 5: Commit

```bash
git add reference/stdlib/algebra.md CHANGELOG.md docs/plan/PLAN-INDEX.md docs/plan/PLAN-138-STDLIB-ALGEBRA-LAWS-AND-PURE-CARRIER-PROOFS.md
git commit -m "TASK-1394: close out phase 138 reference, test handoff, and status"
```

## Dispatch

```
agent: hermes
reasoning: medium
max_turns: 20
toolsets: [terminal, file]
```

## Verification

- [ ] Reference docs reflect source-visible law declarations
- [ ] Law declaration status distinct from proof status
- [ ] CHANGELOG entry added
- [ ] PLAN-INDEX task rows updated
- [ ] Full workspace gates pass
- [ ] Independent Codex review: law names, no fake proofs, Result domain failure, Applicative composition present, generated-test handoff consistent
