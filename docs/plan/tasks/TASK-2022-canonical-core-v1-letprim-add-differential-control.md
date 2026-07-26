# TASK-2022: Canonical Core V1 `LetPrim(Add)` Differential Control

**Status:** Complete
**Phase:** TASK-1988 implementation follow-up; owned by [TASK-439](TASK-439-differential-conformance-harness-rust-first.md)
**Depends on:** TASK-2020, TASK-2021, TASK-439, [Ash Canonical Core](../../spec/CANONICAL-CORE.md), and the existing Core text/parser/validator/typechecker/checked-lowering APIs

## Description

Extend the closed `ash-canonical-core-fixture/v1` private prototype boundary by one independently
fixed pure-primitive control. Its Core text computes `2 + 5`, binds the result, and returns the
bound result through the checked lowering's answer continuation:

```text
(let-prim sum add ((lit-int 2) (lit-int 5)) sum)
    -> CPS LetPrim(sum, Add, [Int(2), Int(5)], Jump(__answer, Var(sum)))
    -> canonical Return(Int(7))
```

This is neither general primitive execution nor a general canonical-Core loader. It is one exact
file-backed artifact that proves the existing canonical Core parse → validate → typecheck →
checked-lower → private-evaluate pipeline retains the primitive binding and answer continuation.

## Authoritative References

- [Ash Canonical Core: `CORE-CPS-SYNTAX-001`](../../spec/CANONICAL-CORE.md#core-and-cps-syntax):
  canonical Core/CPS syntax and private checked boundary.
- [Ash Canonical Core: `SEM-TARGET-CORE-CPS-001`](../../spec/CANONICAL-CORE.md#operational-semantics):
  checked Core/CPS terminal outcomes.
- [Ash Canonical Core: `CONF-IMPLEMENTATION-001`](../../spec/CANONICAL-CORE.md#implementation-conformance):
  rule identity and observable evidence.
- [TASK-2004](TASK-2004-core-cps-production-boundary-decision.md): Core/CPS remains private and
  non-production.
- [TASK-2020](TASK-2020-canonical-core-v1-differential-fixture-adapter.md): closed V1 manifest
  boundary and phase-separated private adapter.
- [TASK-2021](TASK-2021-canonical-core-v1-letval-differential-control.md): exact independent
  `LetVal` control and its shape-coupling precedent.
- [TASK-439](TASK-439-differential-conformance-harness-rust-first.md#canonical-core-v1-bounded-controls-adapter):
  sole corpus/harness owner.

## Scope

### In scope

- One local fixture directory, for example
  `tests/differential/corpus/canonical-core-v1-letprim-add-return-int-7/`, using the unchanged
  closed V1 manifest:

  ```json
  {
    "schema_version": "ash-canonical-core-fixture/v1",
    "case_id": "canonical-core-v1-letprim-add-return-int-7",
    "target": "rust-checked-core-cps-prototype",
    "canonical_rule_ids": ["SEM-CPS-PRIM-001", "SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
    "core_text": "(let-prim sum add ((lit-int 2) (lit-int 5)) sum)"
  }
  ```

  `sum` is a fixture spelling only. The primitive, arity, two literal operands, binder, body, and
  terminal result are all fixed by this one control.
- Reuse the V1 decoder and local `core_text` carrier unchanged. No schema field, environment,
  target spelling, path/URL/input-file carrier, or external Core file is introduced.
- Extend only the private canonical-Core fixture adapter's exact accepted-case set. It must retain
  separate manifest decode, Core parse, Core validation, typecheck, and checked-lowering failures.
- Parse and validate the exact `CoreExpr::LetPrim`, typecheck it in the same closed
  `CoreTypeCheckEnv`, lower it with checked lowering, and inspect the exact CPS shape:
  `LetPrim(sum, Add, [Int(2), Int(5)], Jump(__answer, Var(sum)))`.
- Compare only canonical `Return(Int(7))`; direct runtime is explicitly `Unsupported`.
- Add negative controls proving changed binder, primitive, arity, operand literal/type/form, or
  body fails before terminal comparison. Preserve the existing closed-schema, indirection, path,
  and symlink protections from TASK-2020 and TASK-2021.

### Explicit exclusions

- General `LetPrim`, arbitrary primitives/binders/operands/bodies, nested arithmetic, source
  lowering, effectful primitive operations, or a claim that valid V1 Core terms generally execute.
- V1 schema widening, JSON Core AST, Core file/import/module loading, non-empty environment,
  provider/capability/admission carrier, remote input, or filesystem indirection.
- Production Engine/CLI execution, public Core APIs, provider or handler frames, runtime traces,
  monitors, direct-runtime↔CPS parity, or any change to TASK-2004's retained-private boundary.

## Requirements and Invariants

1. **Same closed V1 boundary.** The third control uses precisely the existing V1 keys and private
   target; its rule labels cannot select alternate semantics.
2. **Exact case-and-shape coupling.** This case ID accepts precisely `sum = add(2, 5); sum` in
   canonical Core text. A changed binder, primitive, operands, arity, body, literal spelling, or
   another Core form cannot reuse this route.
3. **Checked primitive evidence.** Success visibly traverses canonical Core parsing, validation,
   typechecking, checked lowering, and private evaluation. The inspected term is a checked CPS
   `LetPrim(Add)` followed by `Jump(__answer, Var(sum))`, not hand-authored CPS or a source bridge.
4. **Terminal contract.** Only exact `Return(Int(7))` is compared after primitive evaluation.
5. **Containment and fail-closed behavior.** Rejection reaches neither comparator nor direct
   runtime and cannot fall back to source parsing, legacy adapters, JSON CPS-kernel decoding, or
   a generic primitive evaluator.

## TDD Steps

1. Inspect TASK-2020/2021 V1 routing and current Core `LetPrim` text/typecheck/lowering APIs;
   confirm the closed manifest already carries every required datum.
2. Add the fixture and focused engine test, initially requiring its fixed case to load, expose the
   exact checked CPS shape, and privately project `Return(Int(7))`; record RED because prior
   identity admission excludes it.
3. Extend the private adapter minimally with this third fixed identity and structural predicate;
   do not generalize accepted Core or primitive support.
4. Add parsed/checked-term assertions and altered binder/op/arity/operand/body/form negatives,
   asserting phase-local rejection before comparison or direct execution.
5. Run focused TASK-2022 plus TASK-2020/2021 tests, TASK-439/TASK-2005 harness tests, relevant
   ash-core parser/typechecker/lowering tests, formatting and Clippy. After green implementation,
   update this task, TASK-439, `PLAN-INDEX.md`, `CHANGELOG.md`, and semantic traceability, then
   run documentation, traceability, and diff gates.

## Expected Completion Evidence

- One closed V1 manifest with the exact `LetPrim(Add)` Core text reaches parse → validate →
  typecheck → checked lower → private projection and produces only `Return(Int(7))`.
- Tests inspect exact parsed/checked CPS `LetPrim` and answer-`Jump` structure, excluding literal
  shortcuts, source bridges, and hand-authored CPS.
- Altered case identity or program structure rejects before comparison; all predecessor closed
  schema/path/symlink/phase safeguards remain green.
- Direct runtime and all production authority remain unchanged.

## Completion Checklist

- [x] A third strict V1 manifest exists with no schema enlargement.
- [x] Its only accepted Core text is the fixed `LetPrim(Add)` control.
- [x] It traverses parse → validate → typecheck → checked lower → private projection.
- [x] Tests assert exact CPS `LetPrim(Add)` / answer-`Jump` and `Return(Int(7))`.
- [x] Altered structure and all existing malformed/indirection routes fail closed.
- [x] Direct runtime and all production authority remain unchanged.
- [x] Focused/relevant tests, formatting, Clippy, docs/traceability, and diff gates pass.

## Completed exact V1 `LetPrim(Add)` control slice

The closed V1 adapter now admits its third and final independently fixed control:
`canonical-core-v1-letprim-add-return-int-7`.  The unchanged manifest shape carries exactly
`(let-prim sum add ((lit-int 2) (lit-int 5)) sum)` with the private
`rust-checked-core-cps-prototype` target and fixed `SEM-CPS-PRIM-001`,
`SEM-CPS-RETURN-001`, and `CONF-IMPLEMENTATION-001` evidence labels.  It traverses distinct
parse, validation, typecheck, checked-lowering, and private terminal-projection stages.

Its checked CPS evidence is exactly `LetPrim(sum, Add, [Int(2), Int(5)],
Jump(__answer, Var(sum)))`, which projects only `Return(Int(7))`.  Admission is fixed-text as
well as structural: any altered whitespace-normalized AST spelling, including `+2` for `2`,
rejects before parsing.  Altered binder, primitive, arity, operand, annotation, body, literal
form, identity, or evidence labels likewise reject during corpus load before comparison.

TASK-2020's literal and TASK-2021's `LetVal` controls remain predecessor-specific fixed-text
controls; they do not admit malformed or alternate text through later parse/typecheck/lowering
stages.  Together the three controls are a closed per-case V1 set, not a generic Core loader.
All run only through the private checked-CPS prototype; direct runtime remains `Unsupported`, and
no production Core/CPS, Engine, CLI, provider, admission, trace, or monitor authority is added.

Verification evidence: `task_2022_canonical_core_v1_letprim_add_fixture` (5 tests), predecessor
V1 controls, TASK-439/TASK-2005 harness tests, workspace Clippy, formatting, diff,
semantic-traceability, and documentation gates passed.
