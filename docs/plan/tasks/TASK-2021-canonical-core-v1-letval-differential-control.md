# TASK-2021: Canonical Core V1 `LetVal` Differential Control

**Status:** Complete
**Phase:** TASK-1988 implementation follow-up; owned by [TASK-439](TASK-439-differential-conformance-harness-rust-first.md)
**Depends on:** TASK-2020, TASK-439, [Ash Canonical Core](../../spec/CANONICAL-CORE.md), and the existing Core text/parser/validator/typechecker/checked-lowering APIs

## Description

Extend the already-closed `ash-canonical-core-fixture/v1` private prototype boundary by one
structural, file-backed Core control.  Its Core text binds the literal `Int(7)` with `LetVal` and
returns that bound value through the checked lowering's answer continuation:

```text
(let-val value : Int (lit-int 7) value)
    -> CPS LetVal(value, Int(7), Jump(__answer, Var(value)))
    -> canonical Return(Int(7))
```

This is not a new V1 schema, a general Core-program loader, or a production Core/CPS route.  It
is the next exact canonical artifact after TASK-2020's `(lit-int 7)` control, demonstrating that
the same canonical-Core parse → validate → typecheck → checked-lower → private-evaluate pipeline
preserves a local lexical value through the answer continuation.

## Authoritative References

- [Ash Canonical Core: `CORE-CPS-SYNTAX-001`](../../spec/CANONICAL-CORE.md#core-and-cps-syntax):
  canonical Core/CPS syntax and the private checked execution boundary.
- [Ash Canonical Core: `SEM-TARGET-CORE-CPS-001`](../../spec/CANONICAL-CORE.md#operational-semantics):
  checked Core/CPS terminal outcomes.
- [Ash Canonical Core: `CONF-IMPLEMENTATION-001`](../../spec/CANONICAL-CORE.md#implementation-conformance):
  rule identity and observable evidence.
- [TASK-2004](TASK-2004-core-cps-production-boundary-decision.md): Core/CPS remains private and
  non-production.
- [TASK-2020](TASK-2020-canonical-core-v1-differential-fixture-adapter.md): the exact V1
  manifest boundary, strict local-literal carrier policy, and phase-separated private adapter.
- [TASK-439](TASK-439-differential-conformance-harness-rust-first.md#canonical-core-v1-bounded-controls-adapter):
  sole corpus/harness owner.

## Scope

### In scope

- One new local fixture directory, for example
  `tests/differential/corpus/canonical-core-v1-letval-return-int-7/`, with the existing exact V1
  manifest shape:

  ```json
  {
    "schema_version": "ash-canonical-core-fixture/v1",
    "case_id": "canonical-core-v1-letval-return-int-7",
    "target": "rust-checked-core-cps-prototype",
    "canonical_rule_ids": ["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
    "core_text": "(let-val value : Int (lit-int 7) value)"
  }
  ```

  `value` is a fixture spelling, not a general identifier admission decision.  The lexical
  structure and expected terminal `Int(7)` are fixed for this control.
- Reuse the V1 decoder unchanged: no new manifest field, environment carrier, target spelling,
  path/URL/input-file carrier, or external Core file is needed.  The only accepted executable
  input remains manifest-local `core_text`.
- Extend only the private canonical-Core fixture adapter's narrow accepted-case contract so it
  recognizes this second fixed V1 structural case alongside TASK-2020's literal control.  The
  loader must still retain separate manifest-decode, Core parse, Core validation, typecheck, and
  checked-lowering failures.
- Parse the exact text, validate it as a lexical `CoreExpr::LetVal`, typecheck it in the same
  default closed `CoreTypeCheckEnv`, and invoke checked lowering before private terminal
  evaluation.  The focused test must inspect the checked CPS term sufficiently to prove exactly
  `LetVal(value, Int(7), Jump(__answer, Var(value)))`; it must not pass through a hand-authored
  CPS kernel term, source bridge, or literal-only shortcut.
- Compare the exact canonical `Return(Int(7))` projection under the manifest's fixed evidence
  labels.  Direct runtime remains explicitly `Unsupported` for this fixture.
- Add negative load/validation controls showing that the new route rejects altered binder,
  literal, annotation, body variable, or structural form before terminal comparison.  Preserve
  TASK-2020's strict schema/target/rule/unknown-field/path/indirection and symlink protections.

### Explicit exclusions

- General `let` support, arbitrary binders/types/values/bodies, multi-binding terms, recursive
  lets, source lowering, or any claim that every valid V1 Core term is now executable.
- Any V1 schema widening, generic Core JSON AST, Core file/import/module loader, non-empty
  environment, provider/capability/admission carrier, remote input, or filesystem indirection.
- Production `Engine`/CLI execution, public Core APIs, providers, handler frames, runtime traces,
  monitors, direct-runtime↔CPS parity, or a change to TASK-2004's retained-private boundary.

## Requirements and Invariants

1. **Same closed V1 boundary.** The second control uses precisely TASK-2020's V1 keys and target.
   No optional structural manifest field is warranted for a closed lexical `Int` example.
2. **Exact structural identity.** The accepted new Core program is precisely one typed `LetVal`
   binding `value` to `Int(7)` whose body is that bound variable.  A literal return, altered
   binder/value/type/body, or another Core form is not silently treated as this control.
3. **Checked continuation evidence.** The successful route must visibly traverse canonical Core
   parsing, validation, typechecking, checked lowering, and private evaluation.  Its checked CPS
   evidence is `LetVal` followed by `Jump(__answer, Var(value))`, not an unchecked CPS encoding.
4. **Terminal contract.** Only the exact canonical `Return(Int(7))` is compared.  Rule IDs remain
   evidence labels and cannot select alternate semantics.
5. **Containment and fail-closed behavior.** Rejected inputs reach neither a comparator nor direct
   runtime and cannot fall back to source parsing, legacy adapters, or JSON CPS-kernel decoding.

## TDD Steps

1. Inspect TASK-2020's private manifest routing and phase-specific error boundaries; identify the
   exact existing Core text, typecheck, checked-lowering, and terminal projection APIs.  Confirm
   the V1 manifest has no missing structural information for the closed `LetVal` control.
2. Add a focused file-backed engine test and the new manifest fixture.  First require it to load
   as V1 and project `Return(Int(7))` only through the private target; record the expected initial
   failure if the literal-only acceptance rule rejects it.
3. Extend the private V1 adapter minimally to admit the exact second identity/term and to retain
   every TASK-2020 guard.  Do not create a generic accepted-Core predicate.
4. Add structural assertions over the parsed/checked-lowered program: `LetVal(value, Int(7),
   Jump(__answer, Var(value)))`.  Add negative altered-binder/value/type/body/form fixtures or
   manifest overrides and assert phase-local rejection before any terminal/direct execution.
5. Run focused TASK-2021 and TASK-2020 tests, TASK-439/TASK-2005 harness tests, relevant
   ash-core text/typecheck/lowering tests, formatting and Clippy.  After green implementation,
   update this task, TASK-439, `PLAN-INDEX.md`, `CHANGELOG.md`, and semantic traceability with
   only this private `LetVal` control, then run the documentation, traceability, and diff gates.

## Expected Completion Evidence

- A closed V1 manifest with the exact `LetVal` text loads through Core parse, validation,
  typecheck, checked lowering, and private evaluation and produces only `Return(Int(7))`.
- Focused structural assertions prove the resulting CPS `LetVal` and answer `Jump`, excluding a
  literal shortcut or hand-authored CPS carrier.
- Perturbing the binder, annotation, literal, body variable, form, or exact text rejects before
  comparison (text before parsing); TASK-2020's decoder/path/symlink protections remain green,
  and phase-local evidence remains limited to the exact admitted control.
- The direct runtime is unsupported and Engine/CLI/provider/admission/trace/monitor/production
  Core/CPS behavior is unchanged.

## Completed exact V1 `LetVal` control slice

The closed V1 adapter now admits one additional, independently fixed canonical-Core artifact:
`canonical-core-v1-letval-return-int-7`. Its unchanged manifest shape carries only
`(let-val value : Int (lit-int 7) value)` and retains the exact private
`rust-checked-core-cps-prototype` target and the existing return/conformance evidence labels.
It follows the same separate Core parse, validation, typecheck, checked-lowering, and private
terminal-projection stages as the earlier literal control.

The accepted checked term is exactly `LetVal(value, Int(7), Jump(__answer, Var(value)))`, which
projects only `Return(Int(7))`. The predecessor literal `(lit-int 7)` control remains a separate
identity-and-shape pair; this completion does not make its literal rule a shared admission rule or
turn the adapter into a general Core loader. Altered binder, literal, annotation, body variable,
or structural form rejects during corpus load before comparison. The TASK-2020 closed-field,
local-path, symlink, identity, schema, target, rule, and phase protections remain in force.

This predecessor is likewise exact-text admitted: only `(let-val value : Int (lit-int 7) value)`
reaches the Core parser.  Any malformed or alternate spelling rejects before parsing, even if it
would normalize to the same AST.  Its phase-local parse/validation/typecheck/lowering evidence
therefore applies only to that fixed admitted text, not to malformed variants.

Direct runtime is explicitly unsupported for this control. No Engine/CLI route, source fallback,
provider/admission/trace/monitor product, public Core API, or production Core/CPS authority was
added.

Verification evidence: `task_2021_canonical_core_v1_letval_fixture` (3 tests),
`task_2020_canonical_core_v1_fixture` (8), TASK-439 (15), TASK-2005 (17), relevant Core and
workspace Clippy checks, formatting, diff, semantic-traceability, and documentation gates passed.

## Completion Checklist

- [x] A second strict V1 manifest exists with no schema enlargement.
- [x] Its only accepted Core text is the fixed typed `LetVal` control.
- [x] It traverses parse → validate → typecheck → checked lower → private projection.
- [x] Tests assert exact CPS `LetVal` / answer-`Jump` structure and `Return(Int(7))`.
- [x] Altered structure and all existing malformed/indirection routes fail closed.
- [x] Direct runtime and all production authority remain unchanged.
- [x] Focused/relevant tests, formatting, Clippy, docs/traceability, and diff gates pass.
