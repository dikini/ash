# TASK-2020: Canonical Core V1 Differential Fixture Adapter

**Status:** Complete
**Phase:** TASK-1988 implementation follow-up; owned by [TASK-439](TASK-439-differential-conformance-harness-rust-first.md)
**Depends on:** TASK-2004, TASK-439, [Ash Canonical Core](../../spec/CANONICAL-CORE.md), and the existing Core text/parser/validator/typechecker/lowering APIs

## Description

Create the first strict, versioned fixture boundary for an *active canonical Core* case.  A
fixture is a self-contained directory whose manifest has schema
`ash-canonical-core-fixture/v1` and carries its Core program as one local `core_text` literal.
The loader must parse that literal through `ash_core::core_ash_text`, validate it as Core, type
check it in the explicitly declared fixture environment, lower only the checked program, and run
the result only through a private differential-prototype adapter.

This task replaces neither the Phase-202 authority sidecar nor the historical workflow-first
catalog.  In particular, `CANONICAL-CORPUS.json` remains an authority graph, existing
`ash-corpus-case/v1` adapters remain adapters, and `ash-cps-kernel-input/v1` through v4 remain
their own deliberately narrow prototype grammar.  The new input is not Ash source text, not a
path to a `.core` file, not a raw trusted CPS term, and not a new Engine/direct-runtime route.

## Authoritative References

- [Ash Canonical Core: `CORE-CPS-SYNTAX-001`](../../spec/CANONICAL-CORE.md#core-and-cps-syntax):
  canonical Core/CPS vocabulary and the retained-private production boundary.
- [Ash Canonical Core: `SEM-TARGET-CORE-CPS-001`](../../spec/CANONICAL-CORE.md#operational-semantics):
  checked Core/CPS execution and terminal outcomes.
- [Ash Canonical Core: `CONF-IMPLEMENTATION-001`](../../spec/CANONICAL-CORE.md#implementation-conformance):
  stable rule identity and permitted-observable evidence.
- [TASK-2004](TASK-2004-core-cps-production-boundary-decision.md): checked Core/CPS remains a
  private/prototype boundary, not the production `Engine` execution authority.
- [TASK-439](TASK-439-differential-conformance-harness-rust-first.md#canonical-v1-adapter-boundary):
  identifies this versioned canonical-Core fixture/parser/adapter as the prerequisite for any
  genuine canonical case.
- `crates/ash-core/src/core_ash_text.rs`, `core_ash_typecheck.rs`, and `core_ash_lower.rs`:
  existing Core text parse, validation, typecheck, and checked-lowering owners.

## Scope

### In scope

- A fixture-local `canonical-core.json` manifest with exactly this closed V1 shape:

  ```json
  {
    "schema_version": "ash-canonical-core-fixture/v1",
    "case_id": "canonical-core-v1-return-int-7",
    "target": "rust-checked-core-cps-prototype",
    "canonical_rule_ids": ["SEM-CPS-RETURN-001", "CONF-IMPLEMENTATION-001"],
    "core_text": "(lit-int 7)"
  }
  ```

  The implementation may add an explicitly documented, closed `environment` field only when a
  first non-empty Core environment is needed.  V1 starts with the default empty
  `CoreTypeCheckEnv`; it must not accept ad-hoc runtime/provider/environment JSON.
- Exact schema-version and target selection.  `target` is required and must equal the existing
  private `rust-checked-core-cps-prototype` target; a direct-runtime, Engine, public CLI, or
  arbitrary target spelling rejects during fixture load.
- `#[serde(deny_unknown_fields)]` (or an equally strict structural decoder) for the manifest and
  every nested V1 schema record.  Empty IDs, duplicate rule IDs, unsupported rule IDs, unknown
  fields, absent required fields, and non-string `core_text` reject before any Core parse or run.
- The Core text stays in the manifest literal.  No `input_file`, `core_file`, relative/absolute
  path, URL, include, symlink resolution, or filesystem indirection is accepted by V1.  A fixture
  may be loaded from its case directory, but all executable Core input is the parsed literal.
- A private adapter in `ash_engine::differential` (or an equally private harness module) that
  executes this ordered pipeline:

  ```text
  manifest decode/identity validation
      -> parse_core_expr(core_text)
      -> CoreExpr::validate()
      -> type_check_and_lower_core_program(default closed CoreTypeCheckEnv, explicit lowering context)
      -> existing private checked-CPS evaluator
      -> existing canonical terminal projection/comparison
  ```

  Each failure remains a corpus-load/prototype diagnostic and names its phase; it must never fall
  back to source parsing, direct evaluation, unchecked `lower_core_program`, or hand-authored CPS
  decoding.
- One positive literal-control fixture (for example `lit-int 7`) proving the complete
  parse/validate/typecheck/checked-lower/private-evaluate path and its exact canonical terminal
  return projection.  The test must inspect the parsed Core/checked-lowered boundary enough that
  it cannot pass through the existing JSON CPS-kernel parser.
- Negative load tests for bad schema, wrong/missing target, unknown field, empty/duplicate rule
  identity, malformed Core text, invalid Core validation, typecheck failure, lowering failure,
  unsupported rule attribution, and every path/indirection-shaped manifest field.  Each must
  reject before terminal comparison and before direct-runtime execution.
- Keep the fixture relation in the differential harness only.  Surface source parsing, `Engine`,
  CLI, host providers, admission, handler-frame installation, monitors/traces, source lowering,
  and production Core/CPS promotion are non-consumers.

### Explicit exclusions

- Translating, rewriting, or making executable the historical SPEC-001 workflow/`Act`/`Proc`
  catalog; this task does not turn historical material into V1 cases.
- A generic Core JSON AST, a permissive arbitrary Core file loader, embedded source programs,
  include/import/module resolution, sandbox/path policy, or remote fixture loading.
- New Core language forms, Core parser/typechecker/lowering semantics, operation environments,
  provider/handler execution, rows/admission, contracts, traces, monitor products, diagnostics
  equivalence, or source/Core refinement claims.
- Any direct `Engine::run`, public API, CLI command, production scheduler, or broad
  direct-runtime↔CPS parity claim.

## Requirements and Invariants

1. **Canonical input identity.** Only the precise V1 schema and target identify this fixture kind.
   Existing `ash-corpus-case/v1` and `ash-cps-kernel-input/v*` records cannot be reinterpreted as
   canonical Core merely by adding fields.
2. **Literal locality.** `core_text` is the sole Core program carrier.  It is parsed as text, not
   interpreted as a filesystem path; all path-like carrier names and unknown fields are rejected.
3. **No trust jump.** A successful parse is insufficient.  The adapter must validate, type check,
   and use *checked* lowering before private evaluation.  Parsing, validation, typechecking, and
   lowering each retain their distinct error boundary.
4. **Closed environment.** V1 has a deterministic default closed type environment and lowering
   context.  No fixture can smuggle provider state, an operation table, capabilities, imports, or
   a host execution target through untyped JSON.
5. **Rule attribution.** The manifest gives non-empty, unique, currently admitted canonical rule
   IDs and the fixture is accepted only when its observable claim matches the V1 adapter's narrow
   literal-return contract.  Rule strings are evidence labels, never an execution selector.
6. **Prototype containment.** The adapter uses the existing private checked-CPS prototype only;
   it does not change the direct runtime, `Engine` APIs, public representation terminology, or
   TASK-2004’s production-boundary decision.
7. **Fail closed.** No invalid manifest or Core program reaches a terminal comparator, and no
   rejected fixture can obtain an observable through a source/CPS fallback.

## TDD Steps

1. **Freeze interfaces and authority.** Inspect `crates/ash-engine/src/differential.rs`, its
   `LoadedCase` routing and `RustExecutionTarget`, the existing checked-CPS input schemas, and
   `ash_core` text/validation/typecheck/lowering APIs.  Confirm the adapter is private and record
   the one allowed target spelling and the existing terminal normalizer.
2. **RED: V1 literal positive control.** Add
   `crates/ash-engine/tests/task_2020_canonical_core_v1_fixture.rs` and a file-backed fixture
   under `tests/differential/corpus/`.  Require a V1 manifest carrying `(lit-int 7)` to load and
   return the same canonical return projection only through the Core text pipeline.  Prove the
   existing CPS-kernel JSON decoder is not the successful carrier.
3. **GREEN: closed manifest decoder and routing.** Add private V1 manifest structs with denied
   unknown fields, exact schema/target/rule validation, and one explicit corpus routing branch.
   Do not broaden `DirectRuntimeInputFile`, accept an `input_file`, or expose a public loader.
4. **RED/GREEN: checked Core pipeline.** Make the positive fixture fail until it invokes
   `parse_core_expr`, `validate`, `type_check_and_lower_core_program`, and the private checked CPS
   evaluator in order.  Add phase-specific tests so parse, validate, typecheck, and lowering
   failures cannot be confused or bypassed.
5. **RED/GREEN: reject envelope expansion.** Add table-driven malformed-manifest fixtures for
   bad schema/target/rules, unknown fields, and `core_file`/`input_file`/`path`/URL-like attempts.
   Assert every case fails during loading with no direct-runtime call and no terminal projection.
6. **Regression and documentation.** Run the new focused test plus TASK-439, TASK-2005,
   relevant `ash-core` text/typecheck/lowering tests, and all affected Clippy/format checks.  Once
   green, update this task, TASK-439’s canonical-v1 boundary wording, `PLAN-INDEX.md`,
   `CHANGELOG.md`, and semantic traceability only for the private V1 literal-return adapter.
   Run `python3 tools/docs/validate_semantic_traceability.py --root . --graph
   docs/spec/SEMANTIC-TRACEABILITY.json`, `bash scripts/check-docs-gate.sh`, and `git diff --check`.

## Expected Completion Evidence

- A single fixture with `ash-canonical-core-fixture/v1`, the exact private prototype target, a
  closed V1 manifest, and literal `(lit-int 7)` produces the canonical return envelope after
  Core parse, validation, typecheck, and checked lowering.
- Focused regression tests enforce fixed-text admission before parsing, so malformed or alternate
  text cannot reach a comparator or the direct runtime.  Phase-local Core-pipeline evidence is
  restricted to the one exact admitted literal control.
- Strict unknown-field/path-shaped-field tests prove that the only V1 executable carrier is
  manifest-local Core text.
- Public `Engine`, CLI, direct runtime, provider/admission, trace/monitor, and production
  Core/CPS behavior remain unchanged; documentation calls the result a private prototype adapter,
  not general canonical-catalog execution or conformance completion.

## Completed strict V1 literal-control slice

The completed adapter accepts exactly one file-backed canonical-Core control:
`canonical-core-v1-return-int-7`.  Its closed
`ash-canonical-core-fixture/v1` manifest carries the local literal
`(lit-int 7)`, the exact `rust-checked-core-cps-prototype` target, and the
required `SEM-CPS-RETURN-001` / `CONF-IMPLEMENTATION-001` evidence labels.
The private differential adapter runs that literal through distinct Core
parse, validation, typecheck, checked-lowering, and private checked-CPS
evaluation stages before projecting the canonical return envelope.

The manifest decoder denies unknown fields and all path/indirection-shaped
carriers.  The fixture loader rejects a nonlocal case directory or manifest
symlink before decode.  Schema/target/rule/identity failures and separate
parse, validation, typecheck, and lowering failures stop during corpus load;
none can reach terminal comparison, direct runtime, source fallback, or the
legacy CPS-kernel decoder.  The direct-runtime target for this fixture remains
explicitly `Unsupported`.

The later fixed-text controls make the admission ordering explicit: this literal control is also
admitted only when its `core_text` is exactly `(lit-int 7)`.  Altered or malformed text rejects
before Core parsing, including text that could normalize to the same parsed AST.  The retained
phase-local parse/validation/typecheck/lowering evidence concerns only the exact admitted control;
this predecessor does not claim malformed text proceeds to any later phase.

This is private prototype evidence only.  It adds no Engine or CLI execution
path, runtime/provider/admission state, trace/monitor product, or production
Core/CPS authority, and it preserves the legacy adapter routes unchanged.

Verification evidence: `task_2020_canonical_core_v1_fixture` (8 tests),
TASK-439 (15), TASK-2005 (17), relevant ash-core Core parse/typecheck/lowering
tests, Clippy, formatting, diff, semantic-traceability, and documentation
gates all pass; QA and code review found no remaining issue in the bounded
slice.

## Completion Checklist

- [x] Strict `ash-canonical-core-fixture/v1` manifest exists and rejects unknown/invalid fields.
- [x] `core_text` is the only accepted Core carrier; file/path/URL indirection rejects.
- [x] The only V1 target is `rust-checked-core-cps-prototype` and routing stays private.
- [x] Positive fixture reaches parse → validate → typecheck → checked lower → private CPS
  terminal projection in that order.
- [x] Parse/validation/typecheck/lowering and schema/target/rule/path negatives fail closed.
- [x] No Engine/direct runtime, source fallback, public API/CLI, provider/admission, trace/monitor,
  or production-boundary change is introduced.
- [x] Focused/relevant regression, formatting, Clippy, docs/traceability, and diff gates pass.
- [x] TASK-439, plan index, changelog, and traceability describe only the implemented narrow
  private literal-control boundary.
