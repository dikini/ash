# TASK-1506: QuickCheck v1 closeout and review

## Status: 📝 Planned

## Description

Close Phase 151 by reconciling status surfaces, updating CHANGELOG/reference/spec index, running broad verification, and obtaining independent review focused on bridge removal, evidence overclaiming, and final-surface behavior.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- 📝 TASK-1505: QuickCheck v1 final-surface fixtures and docs (planned)

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Mark TASK-1497 through TASK-1506 complete only after verification evidence exists.
2. Update PLAN-151, PLAN-INDEX, SPEC-087 status, docs/spec/README, CHANGELOG, and reference pages if touched.
3. Run broad gates and docs link/trailing-whitespace checks.
4. Run independent review for semantic bridge leakage, evidence overclaiming, and no-Cargo final surface.
5. Patch review findings and rerun focused verification.

### Property Requirements

- All status surfaces agree.
- No retained bridge acts as independent semantic authority.
- Aggregate evidence wording never claims proof.
- Broad gates and docs checks are recorded.

## TDD Steps

### Step 1: Status reconciliation

Audit task files, PLAN-151, PLAN-INDEX, SPEC-087, docs/spec/README, and CHANGELOG.

### Step 2: Verification gates

Run scoped and broad gates appropriate to touched crates.

### Step 3: Independent review

Request review and patch/re-review any blockers.

## Dispatch

```
agent: hermes
reasoning: high
max_turns: 20
toolsets: [terminal, file, coding]
```

## Verification

```
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-cli --test test_command -- --nocapture
  - cargo clippy -p ash-cli --all-targets -- -D warnings
  - git diff --check
checklist:
  - [ ] Focused tests pass and are non-zero
  - [ ] No-Cargo final-surface fixture added where user-facing behavior changed
  - [ ] Negative leakage/fail-closed cases covered where a bridge or error path is touched
  - [ ] CHANGELOG.md updated under [Unreleased]
```

## Dependencies for Next Task

- Closed Phase 151 with verified evidence and review report.
- Handoff for future SmallCheck/proof evidence phases.

## Notes

Do not commit/push unless explicitly directed by the user.
