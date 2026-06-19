# TASK-1590: Define CPS IR core data structures

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Define the shared Target CPS IR data model for Phase 159: `Atom`, `Value`, `Term`, `ContRef`, `EffectOp`, `HandlerClause`, `HandlerChain`, runtime environment frames, and minimal `.cps` fixture scaffolding for core forms.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- None.

## Requirements

### Functional Requirements

1. Add shared CPS IR carriers in a location usable by both interpreter and future lowering/type-checking code.
2. Represent `Lam` and `Cont` as `Value` variants, not `Term` variants.
3. Represent `LetVal`, `LetPrim`, `LetCont`, `Jump`, and `Call` as `Term` variants.
4. Distinguish `ContRef::Label` from `ContRef::Var`; labels must not be ordinary atoms.
5. Add minimal parser/serializer scaffolding for Phase 1 `.cps` fixtures so early round-trip tests do not wait for Phase 6.

### Property Requirements

- Serializing then parsing Phase 1 terms preserves structure.
- Values in term position and labels in data position are rejected.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** `crates/ash-interp/tests/task_1590_cps_ir.rs`

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `crates/ash-core/src/cps.rs`, `crates/ash-core/src/lib.rs`, `crates/ash-interp/src/cps/sexpr.rs`

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

- Provides the core AST and fixture format consumed by TASK-1591 through TASK-1600.

## Notes

This task chooses the initial module placement. If a new file name differs from `cps.rs`, update PLAN-159 and downstream task files in the same change.
