# TASK-1603: Close out Phase 159

**Status:** ✅ Complete
**Phase:** [PLAN-159](../PLAN-159-CPS-IR-INTERPRETER.md)
**Owner:** Phase 159

## Description

Close Phase 159 by reconciling plan/task/changelog status, running focused and broad verification for the isolated prototype interpreter, auditing reference/docs drift, and recording honest remaining deferrals.

## Specification Reference

- [SPEC-098b: Target IR](../../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-096b: Target Effect System](../../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [SPEC-097b: Target Type System](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-095b: Target Grammar](../../spec/SPEC-095b-TARGET-GRAMMAR.md)

## Dependencies

- 📝 TASK-1590 through TASK-1602 complete or explicitly deferred with user approval.

## Requirements

### Functional Requirements

1. Re-read PLAN-159, PLAN-INDEX, all TASK-1590 through TASK-1602 files, SPEC-098b, SPEC-096b, and SPEC-097b.
2. Run focused CPS IR tests and broad affected-crate gates.
3. Run docs link/trailing-whitespace checks over changed plan/spec/task files.
4. Reconcile task statuses, PLAN-159 milestones, PLAN-INDEX progress row, and CHANGELOG.
5. Request or run independent review before marking the phase complete.
6. Preserve explicit deferrals for bytecode, JIT, legacy AST lowering, Lean 4 differential testing, mutual recursion, row polymorphism, effect aliases, and full discharge unless implemented in a later phase.

### Property Requirements

- Status surfaces must agree: task files, PLAN-159, PLAN-INDEX, and CHANGELOG cannot contradict each other.
- No closeout claim may rely on a zero-test run.
- Closeout must not claim legacy lowering or Lean differential testing was implemented by Phase 159.

## Review Remediation Guidance

Phase 159's interpreter is intended to execute CPS IR produced by earlier Ash stages, not to be a general-purpose runtime validator for arbitrary malformed IR. Remediation should therefore preserve a two-layer contract:

```text
Raw .cps text / producer output
        |
        v
CPS parser + validator boundary
        |
        v
Valid CPS program
        |
        v
Lean CPS interpreter
```

The interpreter may assume a `ValidCpsProgram`-style invariant once input has passed the parser/validator boundary. Do not scatter redundant arity, type, label, or row checks through hot evaluator paths merely to make malformed examples stable. Runtime checks are acceptable only when they are needed to avoid Rust panics, represent real dynamic semantics, or guard invariants that cannot be established statically in Phase 159.

The boundary layer must fail closed for malformed hand-authored `.cps` fixtures and for output received from earlier stages during tests. It should own checks such as:

- function call arity equals lambda parameter count;
- primitive operation arity matches the selected `PrimOp`;
- effect raise arity equals the selected operation/handler parameter shape;
- continuation labels and variables resolve within the validated program;
- rows are locally well formed and duplicate-free;
- values, terms, atoms, and continuation references appear only in their allowed syntactic positions.

Concrete implementation guidance for the next development pass:

1. Introduce explicit raw-vs-validated API types or equivalent naming. A preferred shape is `RawCpsProgram(Term)` plus `ValidCpsProgram(Term)`, with `TryFrom<RawCpsProgram> for ValidCpsProgram` returning `CpsValidationError`. If wrapper types are too heavy for this phase, at least expose a clearly named `validate_cps_program(&Term) -> Result<(), CpsValidationError>` and require fixture/load tests to call it before evaluation.
2. Keep `eval` lean by accepting the validated representation or documenting that callers must validate first. Do not treat interpreter arity checks as proof of correctness.
3. Add validator tests for malformed arity, unresolved labels, row duplicates, and kind/position mistakes. These are validator tests, not evaluator semantics tests.
4. Add evaluator tests only for valid programs and semantic behavior: lexical lambda closure capture, continuation environment capture, shallow handler removal before clause execution, provider-frame dispatch/persistence, resume chain restoration, and ordinary successful result observation.
5. Keep `.cps` parser/serializer tests tied to the documented grammar. `parse(serialize(term))` should hold for generated valid terms, and committed lowercase `.cps` fixtures should parse through the same boundary used by the executor.

Review finding classification under this contract:

- `.cps` grammar drift is a boundary blocker because `.cps` is the Phase 159 fixture/load format.
- lambda definition-environment capture is an interpreter semantics blocker.
- shallow handler removal and provider-frame dispatch are interpreter semantics blockers.
- call, primitive, and raise arity are validation-boundary blockers unless a task explicitly promotes a specific dynamic arity check into evaluator semantics.

## TDD Steps

### Step 1: Write tests (Red)

**Files:** Focused CPS IR test commands plus docs validation commands recorded during closeout.

Write focused tests before implementation. Tests must include at least one positive example and one negative or boundary example for this task's contract.

### Step 2: Implement (Green)

**Files:** `docs/plan/PLAN-159-CPS-IR-INTERPRETER.md`, `docs/plan/PLAN-INDEX.md`, `docs/plan/tasks/TASK-1590-*.md` through `docs/plan/tasks/TASK-1602-*.md`, `docs/spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md`, `CHANGELOG.md`

Implement only the slice named by this task. Preserve the SPEC-098b `Atom` / `Value` / `Term` boundary and avoid direct-style convenience nodes.

### Step 3: Integrate

Update all status surfaces in one pass after final verification evidence is available.

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
  - git diff --check -- docs/plan/PLAN-159-CPS-IR-INTERPRETER.md docs/plan/PLAN-INDEX.md docs/plan/tasks CHANGELOG.md
checklist:
  - [ ] Focused CPS tests pass and execute non-zero cases
  - [ ] Broad affected-crate gates pass or pre-existing failures are classified
  - [ ] Plan/task/index/changelog status surfaces agree
  - [ ] Legacy lowering and Lean differential testing remain explicitly deferred
  - [ ] Independent review completed after final diff
```

## Dependencies for Next Task

- Produces the final Phase 159 closeout evidence and status reconciliation.

## Notes

Do not mark Phase 159 complete while any background verification is still running or while a review finding remains unresolved.
