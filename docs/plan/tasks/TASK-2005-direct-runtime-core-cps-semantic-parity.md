# TASK-2005: Direct-Runtime/Core-CPS Semantic Parity

**Status:** In progress — the reusable differential harness now emits a fail-closed
dimension-by-dimension parity report and executes paired literal-return, both a hand-authored v3
integer-addition prototype, source-derived literal and exact lexical-addition bridges, one
case-bound literal-subtraction bridge, one exact nested-binary ANF bridge, one exact computed-binary-let
bridge, two literal
source-derived Boolean-negation witnesses and two exact lexical Boolean-negation witnesses, both literal V4 conditional branches, and source-derived
true/false literal-conditional bridges, primitive-domain-trap, an explicit `TestClock` missing-discharge, and an executable
source-return continuation fixture, plus one exact `time::sleep(0)` standard-profile/private-provider-frame discharge pair. Only their
respective dimensions and direct-runtime↔checked-Core/CPS relation are
`Compared`; all
other required behavior remains explicit unsupported work, not a broad parity claim. The legacy
direct evaluator is differential-only: it is unavailable outside the canonical non-symlink
built-in corpus root and its exact `(case_id, source)` reference allowlist, and never serves a
production Engine, CLI, admission, or application route.
**Phase:** Follow-up from [TASK-1988](TASK-1988-semantic-implementation-deprecation-audit.md)
**Depends on:** TASK-2004

## Description

Establish behavior-level parity, or explicit bounded divergence, between production direct-AST
execution and the canonical Core/CPS relation.

## Requirements

- Compare values, structured traps, frame ordering, missing discharge, rows, continuation use,
  dynamic contracts, and allowed external outcomes.
- Normalize only canonical observable fields; exclude storage/scheduler/provider internals.
- Treat a mismatch as drift, not an implementation-defined semantic amendment.
- Assign every permitted divergence to a canonical rule and owner.

## TDD Steps

1. Add paired direct-runtime/Core-CPS fixtures with expected terminal projections.
2. Add mutation fixtures for outermost dispatch, lost traps, and wrong continuation use.
3. Implement required bridge/refinement behavior or record explicit bounded divergence.
4. Run differential conformance and all affected quality gates.

## Completion Checklist

- [ ] Each listed observable has parity evidence or an owned divergence.
- [ ] No direct runtime result is asserted to conform by vocabulary alone.
- [ ] Mismatch diagnostics identify canonical rule and source fixture.
- [ ] Traceability, docs, and changelog are updated.

## Evidence required

TASK-1988 establishes prototype frame/multiplicity behavior but production uses `ash_core::Expr`;
completion must compare executable outcomes rather than source shapes.

## Current partial parity-report evidence

`ash_engine::differential::DifferentialHarness` now produces a `ParityReport` for every executed
file-backed direct-runtime fixture.  The focused contract test
[`task_2005_semantic_parity_report.rs`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
requires an explicit disposition, canonical-rule identifier, and owner for all eight required
observable dimensions.  It additionally requires the unavailable
`direct-runtime-to-checked-core-cps` relation to be non-passing and owned by TASK-2004.

This is deliberately a **fail-closed reporting slice**. The `phase202-return-unit` fixture now
executes both direct source and a declared checked CPS `Return(7)` term, normalizes both terminal
envelopes, and reports a mismatch as `Failed`; its matching value and execution relation are
therefore genuinely `Compared`. A fixture without a checked-CPS input remains `Unsupported`.
No row below is a permitted semantic amendment: an eventual mismatch remains drift unless a later
decision records a bounded divergence against its canonical owner.

### Hardened differential-only direct oracle

The legacy direct evaluator is not a general harness execution target. `DifferentialHarness`
enables it only when the loaded root is a non-symlink directory whose canonical path equals the
repository's built-in `tests/differential/corpus`, and only for an exact trusted `(case_id, source)`
tuple in its TASK-2005 allowlist (with the separately fixed time-sleep pair). A copied fixture,
any other root, a symlink root, an unknown case, or an altered source yields an `Unsupported`
direct-runtime result before legacy evaluation. The focused untrusted-root control copies a valid
fixture into a temporary root and proves this rejection.

That evaluator remains a private differential reference: no production Engine, CLI, admission, or
application route invokes it. It cannot establish a compatibility fallback, source execution
policy, or general direct-runtime semantics.

### Closed-empty `absorb_sleep` handler parity slice

`phase202-source-absorb-sleep-handler-parity` adds one separately closed handler tuple to the
private differential corpus. Its sole source is the `TestClock::sleep(0)` program with the one
`absorb_sleep` clause `TestClock::sleep(ms, resume) => resume(ms)` and identity `done`; both
targets terminalize `Int(0)`. The direct side is the fixed local derivation
`done(resume(0)) = 0`; the checked side parses/checks the same fixed source, validates its
concrete operation identity, one checked clause, and closed empty residual row, then terminalizes
only an Engine-issued checked-handler *inspection* artifact. It is neither a generic source/CPS
lowerer nor a production handler admission, calls no `Engine::run` or generic evaluator, and
does not create a provider/frame or derive authority from a row.

The corpus admits `source_fingerprint` only for this exact handler case and only when its manifest
equals `sha256:005a6c46e25884ca13762b7cd26e836b2756263f378fd297aa0afc006e8acf89`. The loader computes
SHA-256 over the direct-source bytes **before checked-carrier metadata validation or either target
dispatch**; absent, foreign-case, noncanonical, or mismatched declarations reject. A payload
change reports this exact form after its case-directory path:

    handler-source fingerprint mismatch: expected `<expected>`, actual `<actual>`

The mutation control replaces `resume(ms)` with `ms` without changing the root application and
proves that load-time rejection. This guard locks this one source/root/case/rule tuple; it does
not extend the schema, allow a new handler case, relax the trusted-root gate, or widen the parity
claim.

**Matrix addendum.** In addition to the listed cases, `Values` is `Compared` for this one exact
case under `SEM-EFFECT-HANDLE-001`, and the direct-runtime↔checked-Core/CPS relation is `Compared`
for this one tuple under `SEM-TARGET-CORE-CPS-001`. Its expected terminal projection is also
recorded under `SEM-CPS-RETURN-001`; no other observable dimension changes status.

### Abortive `trap_sleep` structured-trap parity slice

`phase202-source-trap-sleep-handler-terminal` is one separately case-locked private tuple for the
exact local `trap_sleep` source: its only `TestClock::sleep(ms, resume)` clause evaluates
`1 / 0`, never invokes `resume`, and has identity `done`. The direct side is therefore only
the fixed abortive derivation; it is not the generic direct evaluator. The checked side
parse/checks the same source, verifies its concrete operation, one `resume` binder, and closed
empty residual row, obtains only an Engine-issued checked-handler *inspection* artifact, and
terminalizes its sealed checked CPS evidence. It accepts no caller CPS input, source-to-CPS
lowering, provider/frame construction, or production token.

Both sides must produce exactly the canonical V1 terminal envelope:

    {"schema_version": 1, "kind": "trap", "reason": "division by zero"}

`Structured traps` is `Compared` only for this tuple under `SEM-CPS-TRAP-001`; the
direct-runtime↔checked-Core/CPS relation is `Compared` only for this tuple under
`SEM-TARGET-CORE-CPS-001`, with the envelope projection recorded by
`OBS-TARGET-PROJECTION-001`. This does not turn arbitrary handler-body traps, terminal
conversion, CLI output, `Engine::run`, production admission, generic lowering, or a legacy
fallback into a supported route.

**Matrix addendum.** This paragraph amends the matrix's `Structured traps` and
direct-runtime↔checked-Core/CPS `only` cells for this one named tuple; every other listed
unsupported or bounded disposition remains unchanged.

| Observable / relation | Current report disposition | Canonical owner / report rule identifier | Owner | Evidence boundary |
|---|---|---|---|---|
| Values | `Compared` only for `phase202-return-unit`, `phase202-v3-int-add-return-7`, `phase202-source-int-add-bridge-return-7`, `phase202-source-lexical-int-add-bridge-return-7`, `phase202-source-int-sub-bridge-return-5`, `phase202-source-nested-binary-anf-bridge-return-false`, `phase202-source-computed-binary-let-bridge-return-13`, `phase202-source-bool-not-bridge-return-false`, `phase202-source-bool-not-bridge-return-true`, `phase202-source-lexical-bool-not-bridge-return-false`, `phase202-source-lexical-bool-not-bridge-return-true`, `phase202-v4-if-true-return-int-7`, `phase202-v4-if-false-return-int-9`, `phase202-source-if-true-bridge-return-7`, and `phase202-source-if-false-bridge-return-9` | `SEM-CPS-RETURN-001` for the literal-return value; `SEM-CPS-PRIM-001` for the bounded integer primitive values, exact nested-binary and computed-binary-let witnesses, two literal Boolean-negation witnesses, and two exact lexical Boolean-negation witnesses; `SEM-CPS-IF-001` for V4 and source-bridge literal branch selection | TASK-2005 | The literal pair compares its terminal `Return(7)` value. The hand-authored, literal-source, and exact lexical-source add pairs compare `Int(7)` under the primitive rule. The sole subtraction witness is exactly `fn main() -> Int { 7 - 2 }`, which must lower as `LetPrim(Sub, [Int(7), Int(2)]) → Jump(__answer, Var(result))` and compare `Int(5)` only through the private bridge. The sole nested-binary witness is exactly `fn main() -> Bool { (1 + 2) >= (2 * 3) }`, which must lower as ordered `LetPrim(Add) → LetPrim(Mul) → LetPrim(Ge) → Jump(__answer)` and compare `Bool(false)` only through its private bridge. The sole computed-binary-let witness is exactly `fn main() -> Int { do { let __checked_add_result = 99; let computed = (1 + 2) * 3; return computed + 4; } }`, which must preserve `LetVal(__checked_add_result, 99) → LetPrim(Add) → LetPrim(Mul) → LetVal(computed) → LetPrim(Add) → Jump(__answer)` and compare `Int(13)` only through the private bridge. The two literal Boolean witnesses compare `!true → Bool(false)` and `!false → Bool(true)` only through the private bridge. The paired lexical Boolean witnesses are only `fn main() -> Bool { do { let flag = true; return !flag; } }` and `fn main() -> Bool { do { let flag = false; return !flag; } }`, which must respectively lower as `LetVal flag = Bool(true|false) → LetPrim(Not, [Var(flag)]) → Jump(__answer, Var(result))` and compare `Bool(false|true)`. The lexical integer-add pair has the exact checked shape `LetVal x = 2; LetVal y = 5; LetPrim Add(Var x, Var y); Jump(__answer)`. The V4 and source-bridge pairs compare only literal `true` and `false` branch selection. Their terminal `Return`/answer-`Jump` projections are recorded separately below. |
| Structured traps | `Compared` only for `phase202-primitive-domain-trap` | `SEM-CPS-TRAP-001` | TASK-2005 | Direct typed `EvalError::DivisionByZero` and checked CPS `Trap(Custom("primitive-domain"))` normalize to the same canonical trap envelope. Other CPS trap variants fail closed until they have a declared projection. |
| Frame ordering | `Unsupported` | `SEM-EFFECT-LOOKUP-001` | TASK-2005 | TASK-2014 has one real sealed two-provider production witness, but it has no independent direct-runtime target or differential-harness pair; the isolated CPS lookup model/test is likewise not direct-runtime comparison. |
| Missing discharge | `Compared` only for `phase202-missing-declared-operation-discharge` | `SEM-EFFECT-MISSDISCHARGE-001` | TASK-2005 | Explicit source admission preserves typed `CapabilityAdmissionFailure`; checked CPS preserves typed `CpsError::UnhandledEffect`. The pair compares only their shared exact `EffectOp` projection for `TestClock::sleep(Int) -> Null`, never error classes or display text. |
| Rows | `Unsupported` | canonical parent `TYPE-TARGET-ROW-001`; report currently emits provisional `SEM-ROW-ADMISSION-001` | TASK-2005 | The provisional identifier is not yet a canonical traceability node; TASK-2005 must replace it or promote a stable rule before completion. |
| Continuation use | `Compared` only for `phase202-source-return-continuation` | `SEM-CPS-JUMP-001` | TASK-2005 | Source `do { return 42; }` lowers through `Jump(__answer)` and executes inside an affine `LetCont __answer` before comparison with independent direct execution. |
| Dynamic contracts | `Unsupported` | canonical runtime-observable parent `OBS-TARGET-PROJECTION-001`; report currently emits provisional `SEM-DYNAMIC-CONTRACT-001` | TASK-2005 | The provisional identifier is not yet a canonical traceability node; TASK-2005 must select or promote the exact stable rule before completion. |
| Allowed external outcomes | `Compared` only for `phase202-time-sleep-provider-discharge` | `SEM-EFFECT-LOOKUP-001` | TASK-2005 | The exact file-backed `time::sleep(0)` case compares the one admitted standard-profile lookup and `Null` outcome with its one private checked-CPS provider-frame discharge; it is not a general external/provider rule. |
| Direct runtime ↔ checked Core/CPS execution | `Compared` only for `phase202-return-unit`, `phase202-v3-int-add-return-7`, `phase202-source-int-add-bridge-return-7`, `phase202-source-lexical-int-add-bridge-return-7`, `phase202-source-int-sub-bridge-return-5`, `phase202-source-nested-binary-anf-bridge-return-false`, `phase202-source-computed-binary-let-bridge-return-13`, `phase202-source-bool-not-bridge-return-false`, `phase202-source-bool-not-bridge-return-true`, `phase202-source-lexical-bool-not-bridge-return-false`, `phase202-source-lexical-bool-not-bridge-return-true`, `phase202-v4-if-true-return-int-7`, `phase202-v4-if-false-return-int-9`, `phase202-source-if-true-bridge-return-7`, `phase202-source-if-false-bridge-return-9`, and `phase202-time-sleep-provider-discharge` | `SEM-TARGET-CORE-CPS-001` | TASK-2004 | Each listed fixture compares direct execution with either an explicit checked-CPS prototype term or the private source-entry bridge; the retained prototype boundary remains unchanged. |

The report implementation and its contract test are traceable evidence that all required slots are
visible and owned.  They do not discharge `REQ-SEM-TARGET-CORE-CPS-DEFERRED-001`, which remains
the production-parity obligation until executable paired fixtures cover every required observable
with stable canonical rule IDs and mismatch diagnostics.

### Paired v3 `int_add` parity slice

`phase202-v3-int-add-return-7` is a completed, file-backed paired slice. Its direct-runtime input
is `fn main() -> Int { 2 + 5 }`; its other side is the checked-CPS **prototype** input
`LetPrim sum = int_add(2, 5) in Return(Var(sum))`, admitted only by
`ash-cps-kernel-input/v3`. The pair passes only when both normalized terminal envelopes equal
`Int(7)`. It does not change TASK-2004's retained-private production boundary or promote the
checked target to production execution.

| Fact being evidenced | Rule | TASK-2005 disposition and boundary |
|---|---|---|
| The observable `Values` comparison | `SEM-CPS-PRIM-001` | `Compared`: the report assigns the value dimension to the admitted `int_add` computation, not to a generic arithmetic or broad CPS claim. |
| Terminal envelope after the binding | `SEM-CPS-RETURN-001` | The checked prototype separately performs `Return(Var(sum))` and projects the terminal `Int(7)` envelope. This is terminal-return evidence, not a second `Values`-dimension disposition. |
| Cross-target execution relation | `SEM-TARGET-CORE-CPS-001` | `Compared` for this exact direct-runtime/checked-prototype pair only; all unpaired dimensions remain fail-closed as shown in the matrix above. |

The focused contract test
[`paired_v3_int_add_fixture_compares_primitive_values_under_the_primitive_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts the `Values` disposition is `SEM-CPS-PRIM-001` and that the cross-target relation passes.
Terminal `Return` coverage remains separately owned by the checked-CPS v3 corpus evidence in
TASK-439. The slice admits no other primitive, argument form, body form, source-to-CPS lowering,
or general direct-runtime/Core-CPS refinement claim.

### Source-derived atomic-add parity slice

`phase202-source-int-add-bridge-return-7` is a separate, file-backed paired slice with the same
direct source `fn main() -> Int { 2 + 5 }`, but it intentionally contains **no manually authored
checked term**. Its `checked_core_cps` carrier declares only `source_entry: true`, the `values`
dimension, and `SEM-CPS-PRIM-001`; the private engine inspection bridge then lowers the checked
entry to `LetPrim(Add)` followed by the answer `Jump`. The harness admits this exceptional carrier
only when its metadata is complete, the declared rule occurs in that fixture's manifest, and the
source validates as the bounded atomic integer-add form. It compares the resulting normalized
`Int(7)` terminal envelope with independent direct execution.

The loader rejects an absent source, missing or unsupported observable/rule metadata, a rule absent
from the manifest, a non-`LetPrim` source lowering, partial or non-source checked input, and every
`schema_version` on a `source_entry` carrier. These are corpus-load failures, not alternative
execution paths. The earlier metadata-free source-return-continuation fixture remains accepted as
its distinct continuation-use slice; it cannot acquire this primitive-values claim implicitly.

The focused contract test
[`source_int_add_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
requires a passing relation and the precise `SEM-CPS-PRIM-001` values attribution. This is a
private/prototype source-inspection bridge under TASK-2004, not a production Core/CPS execution
route, general source lowering, or a broad parity claim.

`phase202-source-lexical-int-add-bridge-return-7` is a separately manifest-backed source-entry
pair. Its direct source is exactly `do { let x = 2; let y = 5; return x + y; }`; its carrier has
the same complete `values` / `SEM-CPS-PRIM-001` metadata but no authored CPS term or CPS schema.
The private bridge accepts only the exact checked shape `LetVal x = Int(2)` then `LetVal y =
Int(5)` then `LetPrim(Add, [Var(x), Var(y)])` then `Jump(__answer, Var(sum))`, and independently
compares the normalized `Int(7)` terminal envelope. The focused test
[`source_lexical_int_add_fixture_preserves_letval_bindings_before_primitive_value_parity`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts that complete shape as well as the passing primitive-values relation. This admits neither
general lets, arbitrary local values, arbitrary variable arithmetic, general source lowering, nor
production Core/CPS execution; the literal-source atomic-add bridge remains a distinct control.

### Source-derived literal-subtraction parity slice

`phase202-source-int-sub-bridge-return-5` is one closed, file-backed source-entry differential
witness, not a CPS grammar version. Its direct source is exactly `fn main() -> Int { 7 - 2 }`; its carrier has
complete `source_entry: true`, `values`, and `SEM-CPS-PRIM-001` metadata, but no authored CPS term
or CPS schema. The private inspection bridge admits only `LetPrim(Sub, [Int(7), Int(2)]) →
Jump(__answer, Var(result))`, then compares the normalized `Int(5)` result with the
differential-only direct oracle. The focused test
[`source_int_sub_fixture_compares_differential_only_primitive_value_parity`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts that complete spine, its values disposition, and private relation. Corpus-load controls
reject both `2 - 7` and `7 + 2` before either target executes. This is exact private/prototype
evidence under TASK-2004: it admits no variable or arbitrary subtraction, general arithmetic or
source lowering, provider/frame authority, or direct-evaluator fallback. TASK-2004/TASK-2014 now
independently admit atom-only `Sub` through sealed production checked Core/CPS; that separate
production route neither consumes this corpus oracle nor changes this witness's strict case/source
binding or its differential-only direct reference.

### Source-derived exact nested-binary ANF parity slice

`phase202-source-nested-binary-anf-bridge-return-false` is one closed, file-backed source-entry
differential witness, not a general ANF grammar or production admission rule. Its direct source is
exactly `fn main() -> Bool { (1 + 2) >= (2 * 3) }`; its carrier has complete `source_entry: true`,
`values`, and `SEM-CPS-PRIM-001` metadata, but no authored CPS term or CPS schema. Before either
target runs, the private validator binds that case identity and complete source text to exactly
`LetPrim(Add, [Int(1), Int(2)]) → LetPrim(Mul, [Int(2), Int(3)]) → LetPrim(Ge, [Var(add),
Var(mul)]) → Jump(__answer, Var(result))`, with the named dataflow ordering and `Bool(false)`
result. The focused contract test
[`source_nested_binary_anf_fixture_compares_only_its_exact_private_differential_witness`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts that full spine and private values relation. Corpus-load controls reject altered source
text, a changed comparison operator, a changed operand order/value, and a collapsed/nonmatching
`LetPrim` spine before direct or checked execution. This is exact private/prototype differential
evidence only: it does not widen the public nested-binary production family, admit another source
or tree shape to the direct oracle, establish general ANF/parity, or grant a direct-evaluator,
provider, frame, or admission fallback.

### Source-derived exact computed-binary-let parity slice

`phase202-source-computed-binary-let-bridge-return-13` is one closed, file-backed source-entry
differential witness, not a general `let` grammar, ANF grammar, or production-admission rule. Its
direct source is exactly `fn main() -> Int { do { let __checked_add_result = 99; let computed =
(1 + 2) * 3; return computed + 4; } }`; its carrier has complete `source_entry: true`, `values`,
and `SEM-CPS-PRIM-001` metadata, but no authored CPS term or CPS schema. Before either target
runs, the private validator binds the case identity and complete source text to exactly
`LetVal(__checked_add_result, Int(99)) → LetPrim(Add, [Int(1), Int(2)]) → LetPrim(Mul,
[Var(add), Int(3)]) → LetVal(computed, Var(mul)) → LetPrim(Add, [Var(computed), Int(4)]) →
Jump(__answer, Var(result))`, with fixed value flow and `Int(13)` result. The focused contract
test
[`source_computed_binary_let_fixture_compares_only_its_exact_private_differential_witness`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts that complete spine and private values relation. Corpus-load controls reject changed source
text, the deliberate collision binder, an operand value, operand order, the final binding, and an
added source-entry schema before direct or checked execution. This is exact private/prototype
differential evidence only: it does not grant a production route, general local `let` or arithmetic
admission, another direct-oracle source, provider/frame authority, or a direct-evaluator fallback.

### Source-derived Boolean `Not` parity slices

`phase202-source-bool-not-bridge-return-false` and
`phase202-source-bool-not-bridge-return-true` are the two literal, file-backed source-entry pairs for
the exact sources `fn main() -> Bool { !true }` and `fn main() -> Bool { !false }`. Each
`checked_core_cps` carrier declares only `source_entry: true`, the `values` dimension, and
`SEM-CPS-PRIM-001`; neither authors a CPS term or CPS schema. The private inspection bridge
binds the case ID, complete source text, and expected operand as one witness: the false case
accepts only `LetPrim(Not, [Bool(true)]) → Jump(__answer, Var(result))` and compares
`Bool(false)`; the true case accepts only `LetPrim(Not, [Bool(false)]) → Jump(__answer,
Var(result))` and compares `Bool(true)` with the differential-only direct oracle.

The focused contract tests
[`source_bool_not_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
and
[`source_complementary_bool_not_fixture_compares_bridge_derived_primitive_values_under_the_primitive_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
require the matching exact `Not`/answer-jump shape, a passing `SEM-CPS-PRIM-001` values
disposition, and a passing private `SEM-TARGET-CORE-CPS-001` relation. Corpus-load controls reject
both cross-case literal swaps and nested `!!true`/`!!false` forms before either target executes.
This admits no local/variable operands, numeric negation, or other unary form, and no broader
source lowering. It remains a private/prototype differential bridge under TASK-2004: it grants no
production admission, provider/frame authority, async host operation, or direct-evaluator route.

`phase202-source-lexical-bool-not-bridge-return-false` and
`phase202-source-lexical-bool-not-bridge-return-true` are two separately closed source-entry
pairs—not literal witnesses. Their complete sources are respectively
`fn main() -> Bool { do { let flag = true; return !flag; } }` and
`fn main() -> Bool { do { let flag = false; return !flag; } }`; their one admissible binder is
`flag`; and they compare only `Bool(false)` and `Bool(true)`. Their checked bridges must preserve
respectively `LetVal flag = Bool(true) → LetPrim(Not, [Var(flag)]) → Jump(__answer, Var(result))`
and `LetVal flag = Bool(false) → LetPrim(Not, [Var(flag)]) → Jump(__answer, Var(result))` under
`SEM-CPS-PRIM-001`. The focused tests
[`source_lexical_bool_not_fixture_preserves_letval_then_not_before_primitive_value_parity`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
and
[`source_lexical_false_bool_not_fixture_preserves_letval_then_not_before_primitive_value_parity`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
require each case's exact source, binder, spine, result, values disposition, and private relation.
Corpus loading rejects an altered `flag` binding, an unbound case identity, and nested `!!flag` for
each case before either target executes. These are private differential-only evidence: they neither
admit arbitrary lexical Boolean operands nor general `let`/unary source lowering, and grant no
production, provider, frame, or direct-evaluator authority.

`phase202-source-nested-bool-not-bridge-return-true` is a further exact private witness for
`fn main() -> Bool { !!true }`: it compares only `Bool(true)` and requires the complete
`LetPrim(Not, [Bool(true)]) → LetPrim(Not, [Var(result)]) → Jump(__answer, Var(result))` spine.
Source, operand, spine, and schema tampering reject at corpus load. This is differential-only
evidence and neither expands public unary admission nor grants a direct-evaluator fallback.

### Paired V4 literal `If` parity slice

`phase202-v4-if-true-return-int-7` is a completed, file-backed paired slice. Its direct-runtime
input is `fn main() -> Int { if true { 7 } else { 9 } }`; its other side is the checked-CPS
**prototype** input `If(Bool(true), Return(Int(7)), Return(Int(9)))`, admitted only by
`ash-cps-kernel-input/v4`. The pair passes only when both normalized terminal envelopes equal
`Int(7)`. This preserves TASK-2004's retained-private production boundary; it does not promote the
checked target to production execution.

| Fact being evidenced | Rule | TASK-2005 disposition and boundary |
|---|---|---|
| The observable branch-selection value | `SEM-CPS-IF-001` | `Compared`: the `Values` dimension is attributed to selecting the literal `true` branch, not to a generic conditional or terminal-return claim. |
| Terminal envelope after selection | `SEM-CPS-RETURN-001` | The checked prototype separately projects the selected `Return(Int 7)` terminal envelope. This is return evidence, not a second `Values`-dimension disposition. |
| Cross-target execution relation | `SEM-TARGET-CORE-CPS-001` | `Compared` for this exact direct-runtime/checked-prototype pair only; all unpaired dimensions remain fail closed. |

The focused contract test
[`paired_v4_if_fixture_compares_selected_branch_values_under_the_if_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
asserts the `Values` disposition is `SEM-CPS-IF-001` and that the cross-target relation passes.
The V4 corpus control that supplies `Int(1)` as the condition rejects before terminal projection
under TASK-439; it is not a direct-runtime parity result. This slice admits no computed or
non-Boolean condition, broader branch grammar, source-to-CPS lowering, or production Core/CPS
claim.

`phase202-v4-if-false-return-int-9` is the matching false-branch pair. Its direct-runtime input
is `fn main() -> Int { if false then 7 else 9 }`; its checked-CPS **prototype** input is
`If(Bool(false), Return(Int(7)), Return(Int(9)))`. Both normalized terminal envelopes are
`Int(9)`. The `Values` comparison is therefore evidence for `SEM-CPS-IF-001`: it records only
selection of the literal else branch. `SEM-CPS-RETURN-001` remains separate evidence for the
selected `Return(Int(9))` terminal envelope, rather than a second values disposition. The focused
contract test
[`paired_v4_false_if_fixture_compares_the_else_branch_under_the_if_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
requires the same `SEM-CPS-IF-001` values attribution and a passing
`SEM-TARGET-CORE-CPS-001` relation. This is still an exact, private/prototype pair under TASK-2004:
it establishes neither general conditionals nor source lowering nor production CPS execution.

### Source-derived literal `If` parity slice

`phase202-source-if-true-bridge-return-7` and
`phase202-source-if-false-bridge-return-9` are separate file-backed source-entry pairs. Their
direct inputs are exactly `fn main() -> Int { if true then 7 else 9 }` and
`fn main() -> Int { if false then 7 else 9 }`. Neither fixture authors a CPS term or supplies a
CPS schema: each declares only `source_entry: true`, `values`, and `SEM-CPS-IF-001`, which must
also occur in its manifest. The private inspection bridge accepts only the checked shape
`If(Bool(condition), Jump(__answer, Int(7)), Jump(__answer, Int(9)))`; the true fixture projects
`Int(7)` and the false fixture projects `Int(9)`.

The focused tests
[`source_true_literal_if_fixture_compares_the_checked_cps_branch_under_the_if_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
and
[`source_false_literal_if_fixture_compares_the_checked_cps_branch_under_the_if_rule`](../../../crates/ash-engine/tests/task_2005_semantic_parity_report.rs)
require those exact `Bool` and answer-`Jump` branches, passing `SEM-CPS-IF-001` values attribution,
and a passing private relation. The corpus-load negative test rejects an altered literal else
branch before execution. Missing or incomplete source metadata, an absent manifest rule, a
partial/non-source checked input, a schema-versioned source carrier, or any other checked shape
also reject during corpus loading. This does not admit computed/non-Boolean conditions, broader
branches, general conditionals or source lowering, or production Core/CPS execution.

The focused evidence at this slice includes exact corpus-load mutation controls for literal-`If`
branches and Boolean `Not`; related TASK-439 and TASK-2003 evidence remains bounded by their
respective task documents. Their QA evidence includes formatting, clippy, documentation, and
traceability gates. This evidence covers only the bounded prototype surface described here.

### Dynamic-contract pairing boundary

No dynamic-contract pair is currently honest. Direct runtime exposes typed
`ExecError::ContractViolation` and `ExecError::ContractPredicateFault`, but normal source entry
does not install or evaluate a `RuntimeCheckPlan`. Checked CPS has structured contract trap
reasons, while the differential harness deliberately rejects non-custom traps until a canonical
diagnostic serializer exists. The source inspection bridge likewise does not preserve a contract
sidecar or monitor plan. Completing this dimension requires direct monitor wiring, matching
checked Core/CPS sidecar execution, and one shared structured diagnostic projection; comparing
display/debug strings is explicitly out of bounds.

### Missing-discharge paired boundary

`phase202-missing-declared-operation-discharge` is the one bounded missing-discharge pair. It
parses and checks a declaration-backed `TestClock::sleep(0)` source entry, then runs the fixture's
explicit `admission.mode = explicit_missing_discharge` route without a provider binding. The source side stops *before
execution* with typed `ApplicationFailureKind::CapabilityAdmissionFailure`. The same checked entry
lowers to the exact `Raise(TestClock::sleep, Int -> Null)` and, under an empty CPS handler chain,
stops at the distinct runtime error `CpsError::UnhandledEffect`.

Those classes remain deliberately distinct. The fixture compares only the sparse canonical
`missing-discharge` observable whose payload is the complete `EffectOp` identity—namespace,
operation name, operation kind, argument types, and result type. It does not compare error strings,
declare admission and runtime errors equivalent, or project either typed error as a terminal trap.
The relation passes under `SEM-EFFECT-MISSDISCHARGE-001` only for this exact unbound `TestClock`
case. It is not general source-to-CPS capability lowering, handler execution, provider behavior, or
production CPS execution.

Further cases require their own `explicit_missing_discharge` fixture route and independently checked
operation identity. A normal `Engine::run(source)` route does not supply that setup and must not be
treated as this paired evidence.

### Fixed time-provider discharge parity slice

`phase202-time-sleep-provider-discharge` is a separate exact file-backed positive pair. Its direct
reference is only `fn main() -> Null { time::sleep(0) }` with the fixed
`application_default` profile identity and normalized `Null` outcome. It constructs that fixed
profile as a differential reference and returns the constrained `Null` observation; it does not
invoke a host provider. Its checked side is only a private source-entry lowering to
`Raise(time::sleep, Int -> Null)` with the fixture's sole `time_sleep_null` provider-frame
discharge; that private checked execution also projects `Null`.

The fixture fixes the ordered `SEM-EFFECT-LOOKUP-001` and `SEM-EFFECT-RAISE-001` metadata. It
therefore marks `Allowed external outcomes` as `Compared` under the lookup rule and has a passing
private direct-runtime↔checked-Core/CPS relation. The focused
[`task_2005_time_sleep_provider_parity.rs`](../../../crates/ash-engine/tests/task_2005_time_sleep_provider_parity.rs)
also checks the exact `Raise` operation identity, `Int` argument, and `Null` result.

This admits neither a general provider protocol nor frame-ordering parity: the latter remains
`Unsupported`. The checked provider frame and the direct reference both remain private/prototype
evidence under TASK-2004; this slice does not execute a production provider or promote Core/CPS
execution or provider-frame construction to production.
