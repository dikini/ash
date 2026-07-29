# TASK-2023: Canonical Core V1 Literal `If` Differential Controls

> **TASK-2041 status:** The former private controls are historical prototype evidence, not a
> current Engine execution, client-conformance, or fallback route.

**Status:** Complete
**Phase:** TASK-1988 implementation follow-up; owned by [TASK-439](TASK-439-differential-conformance-harness-rust-first.md)
**Depends on:** TASK-2020, TASK-2021, TASK-2022, TASK-439, [Ash Canonical Core](../../spec/CANONICAL-CORE.md), and the existing Core text/parser/validator/typechecker/checked-lowering APIs

## Description

Extend the closed `ash-canonical-core-fixture/v1` private prototype boundary with exactly two
independently fixed literal-conditional controls.  The controls use literal Boolean conditions and
literal integer branches only:

```text
(if (lit-bool true)  (lit-int 7) (lit-int 9))
    -> CPS If(Bool(true),  Jump(__answer, Int(7)), Jump(__answer, Int(9)))
    -> canonical Return(Int(7))

(if (lit-bool false) (lit-int 7) (lit-int 9))
    -> CPS If(Bool(false), Jump(__answer, Int(7)), Jump(__answer, Int(9)))
    -> canonical Return(Int(9))
```

They prove only literal branch selection through canonical Core parse → validate → typecheck →
checked-lower → private evaluation.  They are not a general conditional evaluator, source-lowering
route, or production Core/CPS migration.

## Authoritative References

- [Ash Canonical Core: `CORE-CPS-SYNTAX-001`](../../spec/CANONICAL-CORE.md#core-and-cps-syntax):
  canonical Core/CPS syntax and private checked boundary.
- [Ash Canonical Core: `SEM-TARGET-CORE-CPS-001`](../../spec/CANONICAL-CORE.md#operational-semantics):
  checked Core/CPS terminal outcomes.
- [Ash Canonical Core: `CONF-IMPLEMENTATION-001`](../../spec/CANONICAL-CORE.md#implementation-conformance):
  rule identity and observable evidence.
- [Ash CPS Calculus: `SEM-CPS-IF-001`](../../spec/ASH-CPS-CALCULUS.md#judgments-and-kernel-rules):
  CPS conditional branch selection.
- [TASK-2004](TASK-2004-core-cps-production-boundary-decision.md): Core/CPS remains private and
  non-production.
- [TASK-2020](TASK-2020-canonical-core-v1-differential-fixture-adapter.md): closed V1 manifest
  boundary and phase-separated private adapter.
- [TASK-2022](TASK-2022-canonical-core-v1-letprim-add-differential-control.md): fixed-text,
  per-case acceptance precedent.
- [TASK-439](TASK-439-differential-conformance-harness-rust-first.md#canonical-core-v1-bounded-controls-adapter):
  sole corpus/harness owner.

## Scope

### In scope

- Exactly two local fixture directories using the unchanged closed V1 manifest and private target:

  ```json
  {
    "schema_version": "ash-canonical-core-fixture/v1",
    "case_id": "canonical-core-v1-if-true-return-int-7",
    "target": "rust-checked-core-cps-prototype",
    "canonical_rule_ids": ["SEM-CPS-IF-001", "SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
    "core_text": "(if (lit-bool true) (lit-int 7) (lit-int 9))"
  }
  ```

  The false control uses only `case_id` `canonical-core-v1-if-false-return-int-9` and
  `core_text` `(if (lit-bool false) (lit-int 7) (lit-int 9))`, with the same target and rule IDs.
- Reuse the existing V1 decoder, local `core_text` carrier, fixed-text equality guard, and all
  schema/path/symlink protections unchanged.  No new manifest field or carrier is introduced.
- Extend only the private canonical-Core fixture adapter's exact accepted-case set.  Preserve
  separate manifest decode, Core parse, validation, typecheck, and checked-lowering failures.
- Inspect exact parsed Core `If` structure and checked CPS
  `If(Bool(condition), Jump(__answer, Int(7)), Jump(__answer, Int(9)))`; compare only selected
  canonical `Return(Int(7))` or `Return(Int(9))`.
- Add negative controls for altered condition spelling/value/type/form, branch values/forms/order,
  case identity, rule labels, and text that normalizes to an equivalent AST.  They must reject
  before terminal comparison.

### Explicit exclusions

- General `If`, variable/nonliteral conditions, arbitrary branches, nested conditionals, source
  lowering, effectful conditions/branches, or a claim that valid V1 Core terms generally execute.
- V1 schema widening, JSON Core AST, Core file/import/module loading, non-empty environment,
  provider/capability/admission carrier, remote input, or filesystem indirection.
- Production Engine/CLI execution, public Core APIs, provider or handler frames, runtime traces,
  monitors, direct-runtime↔CPS parity, or any change to TASK-2004's retained-private boundary.

## Requirements and Invariants

1. **Closed two-case boundary.** Only the two stated case IDs, fixed texts, private target, and
   exact rule-label triples are admitted.  `SEM-CPS-IF-001` is evidence for literal branch
   selection; `SEM-CPS-RETURN-001` remains distinct terminal evidence.
2. **Fixed text and shape coupling.** Each control requires exact Core text before parsing, then
   exact parsed Core and checked CPS shapes.  Alternate spellings—including whitespace-normalized
   AST equivalents—cannot reuse either route.
3. **Checked conditional evidence.** Success visibly traverses Core parse, validation,
   typechecking, checked lowering, and private evaluation.  The CPS branches are synthesized
   answer `Jump`s, not hand-authored CPS or a source bridge.
4. **Selected terminal contract.** True projects only `Return(Int(7))`; false projects only
   `Return(Int(9))`.  Neither control compares an unselected branch or asserts a general evaluator.
5. **Containment and fail-closed behavior.** Rejection reaches neither comparator nor direct
   runtime and cannot fall back to source parsing, legacy adapters, JSON CPS-kernel decoding, or a
   generic conditional evaluator.

## TDD Steps

1. Inspect TASK-2020/2021/2022 routing plus the existing Core `If` text/typecheck/lowering APIs;
   confirm the unchanged closed manifest carries every datum needed for both controls.
2. Add the two fixtures and focused engine tests.  Initially require fixed identities/texts to
   load, expose exact checked CPS `If` shapes, and privately project the selected returns; record
   RED because current identity admission excludes them.
3. Extend the private adapter minimally with the two fixed identities and structural predicates;
   do not generalize accepted Core or conditional support.
4. Add parsed/checked-term assertions and alternate condition/branch/spelling/form/identity/rule
   negatives, asserting phase-local rejection before comparison or direct execution.
5. Run focused TASK-2023 plus TASK-2020/2021/2022 tests, TASK-439/TASK-2005 harness tests,
   relevant ash-core parser/typechecker/lowering tests, formatting, and Clippy.  After green
   implementation, update this task, TASK-439, `PLAN-INDEX.md`, `CHANGELOG.md`, and semantic
   traceability, then run documentation, traceability, and diff gates.

## Expected Completion Evidence

- Two closed V1 manifests with the exact literal-`If` texts reach parse → validate → typecheck →
  checked lower → private projection and produce only their selected `Return` values.
- Tests inspect exact parsed Core and checked CPS `If` / answer-`Jump` structure, excluding
  source bridges, hand-authored CPS, literal shortcuts, and general conditional execution.
- Altered identity/text/condition/branches and all predecessor malformed/indirection routes fail
  closed before comparison; the two predecessor V1 controls remain green.
- Direct runtime and all production authority remain unchanged.

## Completion Checklist

- [x] Two strict V1 manifests exist with no schema enlargement.
- [x] Each only accepts its fixed literal-`If` Core text.
- [x] Each traverses parse → validate → typecheck → checked lower → private projection.
- [x] Tests assert exact CPS `If` / answer-`Jump` and selected `Return` evidence.
- [x] Altered structure, normalized-equivalent text, and existing malformed/indirection routes fail closed.
- [x] Direct runtime and all production authority remain unchanged.
- [x] Focused/relevant tests, formatting, Clippy, docs/traceability, and diff gates pass.

## Completed exact V1 literal-`If` control slice

The unchanged closed V1 adapter now admits exactly two further, independently fixed controls:
`canonical-core-v1-if-true-return-int-7` carries only `(if (lit-bool true) (lit-int 7)
(lit-int 9))`, while `canonical-core-v1-if-false-return-int-9` carries only `(if (lit-bool
false) (lit-int 7) (lit-int 9))`. Both retain the private
`rust-checked-core-cps-prototype` target and the ordered evidence labels
`SEM-CPS-IF-001`, `SEM-CPS-RETURN-001`, and `CONF-IMPLEMENTATION-001`.

Each exact text first traverses parse, validation, typechecking, and checked lowering. Its
checked CPS evidence is precisely `If(Bool(condition), Jump(__answer, Int(7)),
Jump(__answer, Int(9)))`; only the selected terminal is projected: true yields
`Return(Int(7))` and false yields `Return(Int(9))`. These branches are synthesized answer jumps
from canonical Core, not source lowering or hand-authored CPS.

Identity, rule label/order, condition value/type/form, branch value/form/order, and alternate
spellings (including whitespace-normalized equivalents) reject during corpus load before parsing
or comparison. The V1 adapter remains a closed per-case admission boundary, not a general
conditional or Core evaluator. Direct runtime remains `Unsupported`; no production Core/CPS,
Engine, CLI, provider, admission, trace, or monitor authority is added.

Verification evidence: `task_2023_canonical_core_v1_literal_if_fixture` (3 tests), predecessor
V1 controls, TASK-439/TASK-2005 harness tests, workspace Clippy, formatting, diff,
semantic-traceability, and documentation gates passed.
