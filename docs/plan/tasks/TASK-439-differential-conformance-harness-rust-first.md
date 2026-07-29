# TASK-439: Differential Conformance Harness (Rust First)

> **TASK-2041 status:** This is a retired historical record of prototype comparison material.
> TASK-2040 removed its Rust differential implementation and tests. It is not current execution
> or conformance evidence and authorizes no executor, conformance route, or fallback. Current
> clients use local Engine instances; the daemon separately executes submitted descriptors and
> manages long-running programs.

## Status: Complete

**Semantic task record:** None — retired historical record; it is outside the active semantic-task scope.

**Semantic coverage map:** [Retired differential material](../SEMANTIC-RULE-COVERAGE.md#retired-differential-material)

**Implementation:** not_implemented
**Evidence:** none
**Parity:** below_spec
**Missing target-spec clauses:** The target conformance domain remains unrealized; a future owner
must establish it without reviving this retired direct-runtime route.

## Semantic workflow record

Rust-first adapters and selected private checked-CPS corpus controls do not realize the target
conformance domain. The tests listed in the record provide confidence only for those controls.

## Description

Build the first differential conformance harness against the canonical semantics corpus, starting with the Rust implementation. This task should turn the Phase 67 contract work into a runnable verification surface: execute canonical IR corpus cases against the Rust runtime, serialize results into the canonical result format, and compare them against expected outcomes or allowed outcome sets where bounded nondeterminism applies.

This is real implementation/test-infrastructure work.

## Specification Reference

- [Ash Canonical Core](../../spec/CANONICAL-CORE.md)
- [SPEC-001: Intermediate Representation](../../spec/SPEC-001-IR.md)
- [SPEC-004: Operational Semantics](../../spec/SPEC-004-SEMANTICS.md)
- [SPEC-021: Runtime Observable Behavior](../../spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
- [SPEC-025: Small-Step Operational Semantics](../../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md)
- [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)
- [TASK-1988: Semantic Implementation and Deprecation Audit](TASK-1988-semantic-implementation-deprecation-audit.md)

## Dependencies

- ✅ [TASK-428: Implementation Conformance Contract](TASK-428-implementation-conformance-contract.md)
- ✅ [TASK-438: Canonical IR Semantics Corpus and Result Format](TASK-438-canonical-ir-semantics-corpus-and-result-format.md)
- ✅ [TASK-433: `ash-interp` Execution-Record Substrate](TASK-433-ash-interp-execution-record-substrate.md)
- ✅ [TASK-435: `Par` Runtime Aggregation Realization](TASK-435-par-runtime-aggregation-realization.md)
- ✅ [TASK-436: Completion-Payload Parity Contract](TASK-436-completion-payload-parity-contract.md)
- ✅ [TASK-437: Retained-Completion Parity Follow-On](TASK-437-retained-completion-parity-follow-on.md)

## Requirements

### Functional Requirements

1. Implement the first differential conformance harness using the Rust implementation as the initial execution target.
2. The harness must:
   - load canonical IR corpus cases,
   - execute them against the Rust runtime/interpreter,
   - serialize runtime results into the canonical result format,
   - compare actual results to expected results or allowed-outcome sets.
3. The harness must handle bounded nondeterminism honestly, especially for `Par`, receive/blocking behavior, and retained completion/control observations where the contract allows multiple valid outcomes.
4. Add tests or harness fixtures demonstrating comparison of at least:
   - exact deterministic cases,
   - allowed-set nondeterministic cases,
   - failure/rejection cases,
   - runtime-observable retained completion/control cases where applicable.
5. Keep the harness extensible so later Lean/reference integration can reuse the same corpus and format instead of inventing a second testing protocol.
6. Update docs/planning/reporting surfaces and `CHANGELOG.md`.
7. Reconcile the older Phase 67 corpus/result references with the active Phase 202 canonical rule
   ids before treating a case as target conformance. The harness must not make historical
   workflow-first semantics a default authority path.
8. Support paired production-direct-runtime and checked Core/CPS execution where TASK-2004 selects
   an executable Core/CPS boundary. Until then, report missing relation support as an owned gap,
   not a passing comparison.

### Non-Functional Requirements

1. Start Rust-first; do not implement Lean execution here.
2. Prefer canonical file-backed corpus fixtures over one-off ad hoc tests.
3. Keep comparison output auditable and useful for debugging mismatches.
4. Mark complete only if the harness provides a real reusable conformance check rather than only one-off example tests.

## TDD Evidence

### Red

Before this task:
- the corpus and result format are planning targets only;
- there is no reusable differential conformance harness for Rust against the Phase 67 semantic contracts;
- future alternate implementations would have no shared comparison substrate.

### Green

This task is complete when:
- Rust can be run against canonical corpus cases through one reusable harness;
- actual results are normalized into the canonical format and checked against expected/allowed outcomes;
- the harness is ready for later Lean/reference extension.

## Files

- Create: `tests/differential/` fixtures and harness files as needed
- Create: `scripts/` support files as needed
- Modify: relevant Rust crate/test infrastructure files as needed
- Modify: `docs/reference/canonical-ir-semantics-corpus.md`
- Modify: `docs/reference/canonical-semantics-result-format.md`
- Modify: `docs/plan/PLAN-INDEX.md`
- Optional Modify: `docs/ideas/README.md`
- Optional Modify: `docs/ideas/IMPLEMENTABILITY-REPORT.md`
- Modify: `CHANGELOG.md`
- Modify: Phase 202 canonical corpus/traceability metadata as needed

## TDD Steps

### Step 1: Write failing harness tests/fixtures

Add corpus fixtures and tests that require Rust results to be normalized and compared against the canonical format.

### Step 2: Implement Rust-first conformance harness

Add the reusable harness, normalization, and comparison logic.

### Step 3: Verify affected crate/test quality

Run at least:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo fmt --check`

### Step 4: Verify GREEN

Expected pass condition:
- the repository now contains a reusable Rust-first differential conformance harness aligned with the Phase 67 contracts.

## Completion Checklist

- [ ] TASK-439 task file created
- [ ] canonical v1 corpus fixtures wired to a harness. Current evidence is deliberately narrower:
  five Phase-202 direct-runtime adapter fixtures under `tests/differential/corpus/`, not the full
  TASK-438 catalog.
- [x] Rust results are normalized into the supported terminal/external subset of the canonical
  result format by `ash_engine::differential`.
- [x] Exact and finite allowed-set comparison are implemented for the supported subset.
- [x] File-backed harness tests cover the exact and allowed-set adapters, including reporting a
  missing Core/CPS relation as `Unsupported`, never as a pass.
- [x] docs/planning/traceability surfaces updated for the partial implementation.
- [x] `CHANGELOG.md` updated.

## Retired historical implementation material and boundary

`crates/ash-engine/src/differential.rs` loads one directory per case beneath
`tests/differential/corpus/`, checks the case/expected metadata and their Phase-202 canonical rule
IDs, runs the declared direct-runtime source fixture, normalizes the resulting value or bounded
external outcome, and compares it against either an exact expected result or a finite allowed set.
`crates/ash-engine/tests/task_439_differential_harness.rs` supplies reusable file-backed evidence
for both comparison modes.

The six current fixtures are adapters, not proof that the historical TASK-438 canonical-IR corpus
is fully executable: `phase202-return-unit` uses an exact paired terminal projection;
`phase202-return-unit-mismatch` deliberately compares direct `7` with checked CPS `Return(8)` and
must report a `Failed` relation with `SEM-CPS-RETURN-001` and both normalized outcomes;
`phase202-bounded-external` uses the explicitly declared `{timeout, unavailable}` provider outcome
set; `phase202-primitive-domain-trap` compares typed divide-by-zero with a checked CPS structured
trap; and `phase202-source-return-continuation` executes the source-lowered affine answer
continuation. They cite active Phase-202 rule IDs rather than making superseded workflow-first
contracts an authority path.

TASK-2005 supplies additional paired `phase202-v3-int-add-return-7`,
`phase202-source-int-add-bridge-return-7`, `phase202-source-lexical-int-add-bridge-return-7`,
`phase202-source-bool-not-bridge-return-false`, `phase202-source-bool-not-bridge-return-true`,
`phase202-source-lexical-bool-not-bridge-return-false`,
`phase202-source-lexical-bool-not-bridge-return-true`,
`phase202-v4-if-true-return-int-7`,
`phase202-v4-if-false-return-int-9`, `phase202-source-if-true-bridge-return-7`,
`phase202-source-if-false-bridge-return-9`, and
`phase202-missing-declared-operation-discharge`, and
`phase202-time-sleep-provider-discharge` adapters through this
same loader. They are parity evidence, not additions to the historical canonical-v1 corpus
or production execution surface.

The paired return, primitive-domain trap, and source-return continuation adapters now have genuine
direct-runtime/checked-Core-CPS relations for their respective observables. Fixtures without a
checked target remain `Unsupported`, while a mismatch remains `Failed`; neither is treated as a
pass. TASK-2004 still retains Core/CPS as private/prototype, and TASK-2005 owns the remaining
parity or declared-divergence work.

### Fixed time-provider discharge adapter

`phase202-time-sleep-provider-discharge` is an exact, file-backed TASK-2005 adapter, not a new
general harness protocol. It admits only the fixed `application_default` direct source
`fn main() -> Null { time::sleep(0) }` and only the private source-entry checked-CPS carrier with
`provider_discharge: "time_sleep_null"`. The direct route and the exact
`Raise(time::sleep, Int -> Null)` checked route both project `Null`; the case fixes
`SEM-EFFECT-LOOKUP-001` and `SEM-EFFECT-RAISE-001` metadata.

The adapter supplies a `Compared` allowed-external lookup observation, while frame ordering remains
unsupported. It neither generalizes provider lookup/frame behavior nor promotes the checked-CPS
prototype target to production execution.

### Canonical-Core V1 controls adapter

TASK-2020 adds the first active canonical-Core fixture without widening this harness into a
general canonical catalog executor. The file-backed
`canonical-core-v1-return-int-7/canonical-core.json` has the exact closed
`ash-canonical-core-fixture/v1` shape and carries `(lit-int 7)` directly in `core_text`.  Its only
accepted target is the private `rust-checked-core-cps-prototype`; the adapter parses, validates,
typechecks, and checked-lowers the Core literal before its private terminal projection.  Direct
runtime is deliberately `Unsupported`.

TASK-2021 adds a separate identity-and-shape pair using that same closed manifest and target:
`canonical-core-v1-letval-return-int-7/canonical-core.json` carries only
`(let-val value : Int (lit-int 7) value)`. It must produce checked
`LetVal(value, Int(7), Jump(__answer, Var(value)))` evidence before the same private
`Return(Int(7))` projection. This lexical control does not share or generalize the predecessor's
literal admission rule: each is exact and independently fail-closed. Direct runtime remains
`Unsupported` for both controls.

TASK-2022 adds a third independent fixed-text control:
`canonical-core-v1-letprim-add-return-int-7/canonical-core.json` carries only
`(let-prim sum add ((lit-int 2) (lit-int 5)) sum)`.  It must produce checked
`LetPrim(sum, Add, [Int(2), Int(5)], Jump(__answer, Var(sum)))` evidence before private
`Return(Int(7))` projection.  The three controls are per-case exact-text admissions, not a shared
structural or parser rule: malformed or alternate text—including spellings that parse to an
otherwise matching AST—reject before parsing.  Thus phase-local parse/validation/typecheck/lower
diagnostics apply only after exact text has been admitted; no predecessor claims that malformed
text reaches a later phase.  Direct runtime remains `Unsupported` for all three.

Unknown fields (including every path/URL/indirection-shaped carrier), nonlocal or symlinked
case/manifest paths, malformed identity/schema/rule metadata, and parse/validation/typecheck/
lowering failures reject at corpus load. TASK-2023 adds two further separately fixed literal-`If`
controls: `canonical-core-v1-if-true-return-int-7/canonical-core.json` carries only
`(if (lit-bool true) (lit-int 7) (lit-int 9))`, and
`canonical-core-v1-if-false-return-int-9/canonical-core.json` carries only
`(if (lit-bool false) (lit-int 7) (lit-int 9))`. Each produces exactly
`If(Bool(condition), Jump(__answer, Int(7)), Jump(__answer, Int(9)))` before projecting only the
selected `Return(Int(7))` or `Return(Int(9))`. Case identity, ordered rule triple, exact text,
and selected result are coupled; alternate condition/branch/form/spelling text rejects before
parsing. Direct runtime remains `Unsupported`; this adds no general conditional execution. The
adapter neither falls back to source/CPS input nor adds Engine, CLI, provider, admission, trace,
monitor, or production Core/CPS execution authority.

The missing-discharge adapter is intentionally different from ordinary direct execution: it alone
uses the fixture-declared `admission.mode = explicit_missing_discharge` route. A source `TestClock::sleep(0)` entry without a
provider binding stops before execution as typed `CapabilityAdmissionFailure`; its checked-CPS
lowering stops at runtime as typed `UnhandledEffect`. The harness projects both only to the exact
structured `EffectOp` identity and compares that sparse `missing-discharge` observable under
`SEM-EFFECT-MISSDISCHARGE-001`. It does not collapse error classes, compare display text, or
establish general source/CPS operation lowering, handler execution, or production CPS behavior.

### Accepted active CPS supersession slice

The first active CPS fixture is
`tests/differential/corpus/cps-kernel-return-int-7/input.ir.json`. Its
`schema_version: "ash-cps-kernel-input/v1"` is the canonical harness input for the
`λAsh-CPS₀` `Return(Int 7)` terminal-observation form. The corresponding case and expected result
cite `SEM-CPS-RETURN-001` and `CONF-IMPLEMENTATION-001`; the harness must compare the exact
canonical return envelope through the distinct `rust-checked-core-cps-prototype` target. The paired
`cps-kernel-return-unbound` fixture requires rejection before terminal comparison, so malformed
canonical CPS input cannot manufacture a result.

The completed next slice is
`tests/differential/corpus/cps-kernel-trap-custom-domain/input.ir.json`: typed
`Trap { reason: { kind: "custom", value: "kernel-custom-domain" } }` must normalize to the exact
canonical structured-trap envelope under `SEM-CPS-TRAP-001` through that same distinct
`rust-checked-core-cps-prototype` target. `cps-kernel-trap-invalid-schema` proves the loader fails
closed: a non-v1 CPS schema is rejected before terminal comparison and cannot manufacture an
observable result. Both fixtures remain prototype evidence under TASK-2004, not production
execution or a general Core/CPS refinement claim.

The completed continuation-use slice is
`tests/differential/corpus/cps-kernel-jump-return-int-7/input.ir.json`. It is deliberately narrow:
the explicit `continuation_store` admits `k(value) = Return(Var(value))` with affine multiplicity
and an empty row, while `Jump(k, Int(7))` also carries an empty row. It must project the exact
terminal return under `SEM-CPS-JUMP-001` and `SEM-CPS-RETURN-001`. The companion
`cps-kernel-jump-absent-continuation` fixture rejects a missing continuation before terminal
comparison. This proves neither arbitrary continuation bodies, nonempty rows, multi-shot behavior,
nor production execution; it remains checked-CPS prototype evidence under TASK-2004.

`ash-cps-kernel-input/v1` is now frozen at `Return`, typed-custom `Trap`, and continuation-store
`Jump` forms; `LetVal` is not retrofitted into that version. The
completed v2 atomic-binding slice is instead
`tests/differential/corpus/cps-kernel-v2-letval-return-int-7/input.ir.json`. Its strict grammar
admits only `LetVal { name, value: Int, body: Return(Var(name)) }`: the bound name must be nonempty,
the value must be an integer atom, and the returned variable must exactly equal the binder. It
projects the exact `SEM-CPS-LETVAL-001`/`SEM-CPS-RETURN-001` terminal return through
`rust-checked-core-cps-prototype`. The companion
`cps-kernel-v2-letval-return-wrong-variable` fixture proves fail-closed validation before any
terminal comparison when the body returns a different variable. This v2 grammar does not admit
general atoms, arbitrary bodies, nesting, rows, continuations, source lowering, or production
execution.

The completed v3 primitive slice is
`tests/differential/corpus/cps-kernel-v3-letprim-int-add-return-7/input.ir.json`. V1 and V2 remain
frozen; v3 admits only `LetPrim` with primitive `int_add`, exactly two integer literals, a bound
name, and `Return(Var(bound name))` body. It projects the exact `Int(7)` return under
`SEM-CPS-PRIM-001` and `SEM-CPS-RETURN-001`. The
`cps-kernel-v3-letprim-unsupported-primitive` companion rejects `int_sub` before terminal
projection. No other primitive, argument kind/arity, body form, row, continuation, or production
execution claim is admitted by this slice.

The completed v4 conditional slice is
`tests/differential/corpus/cps-kernel-v4-if-true-return-int-7/input.ir.json`.
V1, V2, and V3 remain frozen. V4 admits only one `If` term with a literal `Bool` condition and
two `Return` branches whose values are literal `Int`s; it carries no continuation store and lowers
with an empty row. The `true` fixture selects `Return(Int 7)` rather than `Return(Int 9)`, yielding
the exact terminal return under `SEM-CPS-IF-001` and `SEM-CPS-RETURN-001` through the same
`rust-checked-core-cps-prototype` target. The companion
`cps-kernel-v4-if-nonboolean-condition` fails closed before projection: `Int(1)` in condition
position is invalid CPS input and cannot manufacture a terminal result. V4 does not admit variable
or computed conditions, non-`Int` branch returns, nesting, arbitrary terms, rows,
continuations, source lowering, or production execution.

TASK-2005 additionally pairs the same V4 grammar with one direct false-branch fixture:
`phase202-v4-if-false-return-int-9` compares direct `if false then 7 else 9` with checked
prototype `If(Bool(false), Return(Int(7)), Return(Int(9)))`, observing `Int(9)`. Its `Values`
disposition is `SEM-CPS-IF-001`; `SEM-CPS-RETURN-001` remains terminal-envelope evidence for the
selected branch. This pair remains under TASK-2004's private/prototype boundary and does not add
general conditionals, source lowering, or production execution to the TASK-439 CPS corpus claim.

The separate `phase202-source-int-add-bridge-return-7` literal control,
`phase202-source-lexical-int-add-bridge-return-7` lexical pair,
`phase202-source-int-sub-bridge-return-5` literal subtraction witness, and
`phase202-source-nested-binary-anf-bridge-return-false` nested-binary witness, and
`phase202-source-computed-binary-let-bridge-return-13` computed-binary-let witness are not CPS grammar
versions. Their
file-backed `checked_core_cps` inputs carry no manual term and no CPS schema: they
declare `source_entry: true` plus the exact `values` / `SEM-CPS-PRIM-001` observable claim. The
harness permits only the literal `Int + Int` control or the exact lexical source `let x = 2; let
y = 5; return x + y`: the latter must lower to `LetVal x`, `LetVal y`, `LetPrim(Add, Var(x),
Var(y))`, then `Jump(__answer)`, and the manifest must declare the primitive rule. Missing source
or metadata, an absent manifest rule, any other lowering shape, partial/non-source input, or any
source-entry schema version rejects during corpus load. The subtraction witness instead permits
only the exact source `fn main() -> Int { 7 - 2 }` and `LetPrim(Sub, [Int(7), Int(2)]) →
Jump(__answer, Var(result))`, producing `Int(5)`; swapped operands (`2 - 7`) or `Add` reject at
corpus load before either target executes. The metadata-free continuation adapter is not thereby
reclassified. All three remain differential-only private/prototype TASK-2004 evidence rather than
general lets/arithmetic/subtraction, general source lowering, or a direct-evaluator fallback.
TASK-2004/TASK-2014 separately admit atom-only `Sub` in sealed production checked Core/CPS; that
route does not consume the direct oracle, relax this corpus allowlist, or establish a general
differential claim.

The nested-binary witness instead permits only the exact source
`fn main() -> Bool { (1 + 2) >= (2 * 3) }` and one private ordered
`LetPrim(Add, [Int(1), Int(2)]) → LetPrim(Mul, [Int(2), Int(3)]) →
LetPrim(Ge, [Var(add), Var(mul)]) → Jump(__answer, Var(result))` spine, producing `Bool(false)`.
Source-text, primitive-operator, operand, and nonmatching-spine tampering reject at corpus load
before either target executes. It is distinct from the sealed production binary slice: this
case/source-bound direct-oracle evidence cannot admit another tree, widen production lowering, or
become a direct-evaluator fallback.

The computed-binary-let witness instead permits only the exact source
`fn main() -> Int { do { let __checked_add_result = 99; let computed = (1 + 2) * 3; return
computed + 4; } }` and one private value-flow spine
`LetVal(__checked_add_result, Int(99)) → LetPrim(Add, [Int(1), Int(2)]) → LetPrim(Mul,
[Var(add), Int(3)]) → LetVal(computed, Var(mul)) → LetPrim(Add, [Var(computed), Int(4)]) →
Jump(__answer, Var(result))`, producing `Int(13)`. Source text, the collision binder, operand
value, operand order, final binding, and source-entry schema tampering reject at corpus load
before either target executes. It is distinct from all production lowering: this exact private
case/source-bound direct-oracle evidence cannot admit another local `let` source, widen production
admission, or become a direct-evaluator, provider, or frame fallback.

`phase202-source-bool-not-bridge-return-false` and
`phase202-source-bool-not-bridge-return-true` are likewise the two literal witnesses, not CPS grammar versions. Their
file-backed source-entry carriers have no manual CPS term and declare only `values` and
`SEM-CPS-PRIM-001` for their respectively exact `!true` and `!false` sources. A closed witness
table binds each case identity, complete source text, and `Not` operand before lowering: the
former accepts only `LetPrim(Not, [Bool(true)]) → Jump(__answer, Var(result))` and compares
`Bool(false)`; the latter accepts only `LetPrim(Not, [Bool(false)]) → Jump(__answer, Var(result))`
and compares `Bool(true)` with the differential-only direct oracle. Cross-case literal swaps and
nested forms reject at corpus load before execution. Local/variable operands, numeric negation,
all other unary forms, general source lowering, and production Core/CPS execution remain excluded.

`phase202-source-lexical-bool-not-bridge-return-false` and
`phase202-source-lexical-bool-not-bridge-return-true` are separately closed lexical witnesses,
not literal witnesses or CPS grammar versions. Their file-backed source-entry carriers have no
manual CPS term and are bound respectively to complete source
`fn main() -> Bool { do { let flag = true; return !flag; } }`, binder `flag`, result
`Bool(false)`, and complete source `fn main() -> Bool { do { let flag = false; return !flag; } }`,
binder `flag`, result `Bool(true)`. Before comparison, the private validator accepts only,
respectively, `LetVal flag = Bool(true) → LetPrim(Not, [Var(flag)]) → Jump(__answer, Var(result))`
and `LetVal flag = Bool(false) → LetPrim(Not, [Var(flag)]) → Jump(__answer, Var(result))` under
`SEM-CPS-PRIM-001`. An altered binding, unbound identity, or nested `!!flag` for either case
rejects at corpus load before either target executes. This is differential-only private/prototype
evidence: it does not admit general lexical/unary lowering, production Core/CPS execution,
provider/frame authority, or a direct-evaluator fallback.

The separate `phase202-source-if-true-bridge-return-7` and
`phase202-source-if-false-bridge-return-9` source-entry pairs likewise are not CPS grammar
versions. Each declares only complete manifest-backed `source_entry: true`, `values`, and
`SEM-CPS-IF-001` metadata for the exact source `if true/false then 7 else 9`, with neither a
manual term nor a CPS schema. The private bridge admits only
`If(Bool(true|false), Jump(__answer, Int(7)), Jump(__answer, Int(9)))`, selecting `Int(7)` or
`Int(9)` respectively. Corpus loading rejects altered literal branches as well as absent/incomplete
metadata, an absent manifest rule, partial/non-source input, schema-versioned source carriers, or
every other lowered shape. This is exact source-derived prototype evidence, not general
conditionals, source lowering, or production Core/CPS execution.

The related focused evidence is 51 TASK-2005 tests (including paired lexical binding, identity,
nested-`Not`, and computed-binary-let corpus-load controls), 15 TASK-439 harness tests, and 15 TASK-2003 source-bridge tests; prior QA also records
formatting, clippy, documentation, and traceability gates. These counts do not widen the
prototype claim.

This is an accepted supersession decision for harness inputs: legacy SPEC-001 workflow IR v1
(`Let`/`Seq`/`Ret`, `Act`, `Receive`, and related forms) is formally superseded as a TASK-439
harness-input contract. It remains historical/reference material only and is not accepted by the
active CPS fixture loader. This does not supersede the historical document itself, promote checked
CPS to production execution, establish a general source/Core lowering, or make the whole v1
catalog executable. `rust-checked-core-cps-prototype` remains the private/prototype target selected
by TASK-2004.

No current fixture establishes receive/blocking behavior, retained completion, control/tombstone
observations, runtime traps/rejections, or the remaining canonical v1 cases. Completion therefore
also requires those runtime-observable fixtures, full corpus loading/adaptation, diagnostics for
mismatches, and the wider workspace quality gate specified above.

### Canonical-v1 adapter boundary

No historical canonical-v1 catalog case is currently loadable without changing its authority. The
Phase-202 `CANONICAL-CORPUS.json` is an authority graph, not executable case data. The documented
v1 catalog's legacy SPEC-001 workflow IR (`Let`/`Seq`/`Ret`, `Act`, `Receive`, and related forms)
is formally superseded as TASK-439 harness input; it remains historical/reference material. The
harness accepts its private `ash-phase202-direct-runtime-input/v1` source carrier and, for the
active CPS form only, `ash-cps-kernel-input/v1`. Rewriting a historical workflow case as a
source string would still make that source syntax authoritative instead of the required IR.

Before a genuine canonical case can be added, the project needs a versioned canonical-IR fixture
schema and parser, concrete catalog artifacts, and an executor or validated lowering adapter that
preserves terminal projections. That boundary must also reconcile legacy Workflow/Act/Proc forms
with the active Phase-202 Canonical Core disposition before any result is described as canonical
conformance.

## Dependencies for Next Task

This task outputs:
- the first reusable differential conformance harness for Ash.

Required by:
- TASK-440: Lean Reference Refresh Plan Against Current Semantic Corpus

## Notes

Important constraints:
- Keep comparison semantics driven by TASK-428 and TASK-438, not by test convenience.
- TASK-1988 assigns this task, rather than a new parallel harness, as the sole owner of canonical
  differential corpus/harness work. It must bridge legacy corpus material only through explicit
  canonical rule mappings and status labels.
- Make nondeterminism explicit rather than hiding it with flaky tests.
- Prefer reusable normalization/comparison code over fixture-specific assertions.
