# TASK-1497: QuickCheck v1 live syntax and seam audit

## Status: 📝 Planned

## Description

Audit the live parser, surface AST, stdlib `.ash` syntax, interface evidence support, Phase 150 runner bridges, and law-cache seams before any Phase 151 implementation.

## Specification Reference

- [SPEC-087: QuickCheck v1 Ordinary Strategy Semantics](../../spec/SPEC-087-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [PLAN-151: QuickCheck v1 Ordinary Strategy Semantics](../PLAN-151-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)
- [DESIGN-NOTE: QuickCheck v1 Ordinary Strategy Semantics](../../design/DESIGN-NOTE-QUICKCHECK-V1-ORDINARY-STRATEGY-SEMANTICS.md)

## Dependencies

- None. This is the Phase 151 audit gate.

## Deferral / Planned-Feature Reconciliation

| Prior item | Source | Original reason | Prereqs now? | Decision | Gate |
|---|---|---|---|---|---|
| Phase 150 metadata strategy bridge | [PLAN-150](../PLAN-150-QUICKCHECK-ARBITRARY-STRATEGY.md) | Parser/evidence substrate was not ready for ordinary strategy values | Re-audit in TASK-1497 | remove or quarantine as compatibility shim | negative leakage test proves it is not independent semantic authority |
| Runner-owned primitive/container defaults | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | First-slice MVP fallback | Replaced by ordinary in-scope `Arbitrary<A>` evidence | implement now | missing import/evidence fails closed |
| Batch generation sketches | [SPEC-086](../../spec/SPEC-086-QUICKCHECK-ARBITRARY-STRATEGY.md) | Early design before Strategy discussion | Superseded by `GenContext -> A`; SmallCheck owns enumeration | implement now | generated case trace shows one value per context |

## Requirements

### Functional Requirements

1. Inspect live `.ash` examples and parser contracts for callable fields, type aliases/records, interface method syntax, imports, proof blocks, and `by test` parsing.
2. Locate Phase 150 metadata bridge/fallback code paths and classify each as remove, quarantine, or prerequisite blocker.
3. Locate law/property runner and cache schema seams for seed/run/aggregate records.
4. Patch downstream TASK-1498 through TASK-1506 with exact focused test commands if the audit discovers drift.

### Property Requirements

- No implementation task may rely on invented Ash syntax.
- Every bridge retained after this task has a named compatibility scope and future removal gate.

## TDD Steps

### Step 1: Inspect live syntax and files

**Files:** `std/src/**/*.ash`, `crates/ash-parser/src/surface.rs`, `crates/ash-parser/src/parse_module.rs`, `crates/ash-cli/src/test_runner/**`

Record exact accepted syntax and runner seams.

### Step 2: Bridge inventory

List all metadata/fallback strategy paths and assign remove/quarantine/defer decisions.

### Step 3: Patch follow-on tasks if needed

Replace placeholder verification commands in downstream tasks with exact non-zero commands discovered by the audit.

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

- Exact implementation seam map for TASK-1498 through TASK-1506.
- Bridge inventory and removal/quarantine plan.

## Notes

This is a hard gate. Do not start Rust implementation before this audit has concrete live-code evidence.
