# TASK-1600: Harden the CPS S-expression serializer

**Status:** 📝 Planned
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Extend the Phase 1 serializer scaffold to print every Phase 159 IR form in the canonical `.cps` format accepted by TASK-1599.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1599: Harden the CPS S-expression parser.

## Requirements

### Functional Requirements

1. Serialize every Phase 159 IR form.
2. Use a canonical format for row items, handler clauses, continuation references, records, and tuples.
3. Ensure serializer output parses back through TASK-1599.
4. Add snapshot or golden tests for representative programs.

### Property Requirements

- parse(serialize(term)) equals term for generated Phase 159 terms.
- serialize(parse(text)) normalizes to a stable canonical text for golden fixtures.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1600_cps_ir.rs`

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-interp/src/cps.rs`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Wire the new slice through crate exports and the Phase 159 `.cps` fixture path without replacing the existing workflow interpreter.

## Dispatch

```yaml
agent: hermes
reasoning: medium
max_turns: 15
toolsets: [terminal, file]
```

## Verification

```yaml
strictness: clean
commands:
  - cargo fmt --check
  - cargo test -p ash-core -p ash-interp
  - cargo clippy -p ash-core -p ash-interp --all-targets -- -D warnings
checklist:
  - [ ] Focused tests execute non-zero cases
  - [ ] `.cps` fixtures parse or are explicitly deferred by this task
  - [ ] CHANGELOG.md updated when this task is completed
```

## Dependencies for Next Task

- Provides the serializer half of the differential-test fixture contract.

## Notes

Keep examples normalized CPS IR. Values must be bound with `LetVal`; primitive computations must be bound with `LetPrim`; branch bodies must be `Term`s.
