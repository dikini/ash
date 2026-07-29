---
id: plan.ash.core-lazy-memo-modes
title: Core Lazy and Memo Computation Modes
kind: plan
audience: [human, agent]
authority: design
status: complete
stability: alpha
owner: language
last_verified: 2026-06-21
verified_against:
  specs:
    - docs/spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
    - docs/spec/SPEC-099-CORE-LANGUAGE.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md
---

# Core Lazy and Memo Computation Modes Implementation Plan

> **TASK-2041 status:** The historical CPS runtime references in this completed plan do not
> authorize a current executor or fallback. Current executable routes use local Engine instances.

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement SPEC-101 lazy and memo computation modes for Core Ash, including explicit Core mode carriers, type checking, CPS lowering, runtime force semantics, memo behavior, examples, fixtures, and closeout documentation.

**Architecture:** Extend the existing Phase 161/162 Core Ash pipeline in thin vertical slices: AST/text/validation first, type checking second, CPS thunk runtime support third, then Core-to-CPS lowering and end-to-end fixtures. The initial implementation must preserve creation-time handler/provider-chain capture for thunks and must not add new CPS tail-term variants unless a task proves existing terms cannot preserve the spec.

**Tech Stack:** Rust 2024, `ash-core` Core AST/text/validate/typecheck/lower modules, `ash-interp` CPS interpreter for force and memo runtime behavior, focused integration tests in `crates/ash-core/tests/task_166x_*.rs` and `crates/ash-interp/tests/task_166x_*.rs`, `.core`/`.cps` fixtures under existing fixture directories.

---

## Phase: 163

## Status

Completed: 14/14 tasks complete.

## Background

SPEC-101 defines `Strict`, `Lazy`, and `Memo` mode types, mode-aware Core forms (`Thunk`, `LetMode`, `Force`), lazy re-run semantics, memo cache semantics, re-entrant memo rejection, force row accounting, and CPS lowering through a value-level `ThunkClosure` carrier with captured handler/provider chain.

Phase 161 provides Core AST/text/validation/lowering. Phase 162 provides annotation-led Core type checking and checked lowering. Phase 163 should build on those boundaries without changing surface-Ash lowering or adding optimizer behavior.

## Scope

### In scope

1. Core AST carriers for mode types, thunk values, `LetMode`, and `Force`.
2. `.core` text parser/serializer syntax and fixture round-trips for all mode forms.
3. Core validation for mode/type agreement, thunk shape, force shape, and representation invariants.
4. CPS value-level `ThunkClosure` carrier and process-local memo state scaffolding.
5. CPS runtime force semantics for lazy and memo thunks, including cached terminal outcomes and re-entrant memo rejection.
6. Core type checking for mode type well-formedness, mode invariance, thunk latent rows, `LetMode`, and `Force`.
7. Public summaries preserving mode and latent-row facts.
8. Core-to-CPS lowering for thunk construction, strict `LetMode`, lazy/memo `LetMode`, and `Force`.
9. Lowering tests for captured handler/provider-chain authority.
10. End-to-end examples and fixtures proving lazy re-run, memo single-run, cached failure/trap replay, row accounting, and mode mismatch diagnostics.
11. Reference documentation and closeout audit.

### Out of scope

| Item | Reason |
|------|--------|
| Surface Ash syntax and surface-to-Core lowering for lazy/memo | Future elaboration phase; this phase starts at Core Ash. |
| Lazy pattern matching, lazy record fields, or per-field modes | Explicitly non-goals in SPEC-101. |
| Optimizations converting lazy to memo or moving force sites | Requires proof of purity/termination and optimizer policy. |
| Persistent cross-run memo caches | SPEC-101 memo cells are process-local runtime state. |
| Parallel forcing or black-hole semantics | Initial behavior is deterministic re-entrant force rejection. |
| New CPS tail-term variants | Initial design uses value carriers plus existing tail terms. |

## Implementation Notes

- Use the implementation decisions below as the contract for Phase 163. Task agents should not
  choose alternate syntax, AST shapes, or force/runtime seams without first updating this plan and
  the dependent task files.
- Keep Core mode carriers in `crates/ash-core/src/core_ash.rs`.
- Keep parser/serializer changes in `crates/ash-core/src/core_ash_text.rs` and fixture tests near Phase 161 tests.
- Keep static mode checks in `crates/ash-core/src/core_ash_typecheck.rs`; do not infer unannotated thunk rows.
- Add runtime thunk behavior in CPS data/interpreter layers (`crates/ash-core/src/cps.rs` and `crates/ash-interp/src/cps/`).
- Lower `Force` through the existing CPS `Term::LetPrim` shape with a dedicated force primitive that restores the thunk's captured handler/provider chain for the thunk body while binding the result into the force-site continuation body.
- Use checked lowering for row-sensitive integration tests wherever type information is needed.
- Add concrete examples before broad property tests. Property tests should target repeated force/memo invariants only after deterministic examples pass.

## Implementation Decisions

These decisions close the Phase 163 handoff gaps. They are intentionally concrete so a smaller
implementation agent can work task-by-task without inventing incompatible representations.

Decision priority for this phase is:

1. Stability of public and cross-crate interfaces.
2. Lack of ambiguity for task agents.
3. Runtime performance.
4. Future extensibility.

When those goals conflict, choose the earlier item. For example, `MemoCellId` keeps a private field
with an explicit constructor/accessor instead of exposing its raw integer: that is stable and
unambiguous, even though direct field access would be shorter.

### Core AST Shapes

Add the following Core carriers in `crates/ash-core/src/core_ash.rs`:

```rust
pub enum CoreEvalMode {
    Strict,
    Lazy,
    Memo,
}

pub enum CoreThunkMode {
    Lazy,
    Memo,
}

pub struct CoreCaptureSet {
    pub values: Vec<CoreName>,
}
```

Extend `CoreType` with one mode wrapper:

```rust
CoreType::Mode {
    mode: CoreEvalMode,
    inner: Box<CoreType>,
    latent_row: Option<CoreRow>,
}
```

`Strict` mode types require `latent_row == None`. `Lazy` and `Memo` mode types require
`latent_row == Some(row)` once TASK-1665 type well-formedness is implemented. This is the
Phase 163 choice for SPEC-101's "where the row is part of the mode type, function summary, or
associated obligation metadata" allowance: the Core implementation stores exported lazy/memo
latent rows on the mode type and mirrors them in public summaries. Thunk values remain the source
of truth for the body row; mode type rows must match the thunk row when the checker has both.

Extend `CoreValue` and `CoreExpr` with these exact shapes:

```rust
CoreValue::Thunk {
    mode: CoreThunkMode,
    result_ty: CoreType,
    body: Box<CoreExpr>,
    row: CoreRow,
    captures: CoreCaptureSet,
}

CoreExpr::LetMode {
    name: CoreName,
    mode: CoreEvalMode,
    ty: CoreType,
    expr: Box<CoreExpr>,
    body: Box<CoreExpr>,
}

CoreExpr::Force {
    name: CoreName,
    thunk: CoreAtom,
    body: Box<CoreExpr>,
}
```

`CoreCaptureSet` is static metadata for validation, diagnostics, and tests. Runtime handler/provider
chain capture happens during CPS lowering/runtime construction, not as user-visible Core data.

Core text does not serialize capture metadata. Parser behavior is fixed:

- parsing `(thunk lazy Type Row Expr)` or `(thunk memo Type Row Expr)` sets
  `captures: CoreCaptureSet { values: vec![] }`;
- serializer always emits `(thunk Mode Type Row Expr)` and omits `captures`;
- non-empty `CoreCaptureSet.values` is allowed only in programmatic AST tests and diagnostics, never
  in `.core` fixture text;
- validator treats `captures` as metadata and must not reject a thunk only because
  `captures.values` is empty.

`CoreValue::Thunk.result_ty` is the strict inner result type `A`. It is never itself a
`CoreType::Mode`. Validation/type checking must reject a thunk value whose `result_ty` is
`Strict A`, `Lazy A`, or `Memo A`; the checker computes the wrapper type
`CoreType::Mode { mode: Lazy|Memo, inner: A, latent_row: Some(row) }` from the thunk's `mode` and
`row`.

### Core Text Syntax

TASK-1661 must implement this fixture syntax exactly:

```text
Type ::= ...
       | (strict Type)
       | (lazy Type Row)
       | (memo Type Row)

Value ::= ...
        | (thunk lazy Type Row Expr)
        | (thunk memo Type Row Expr)

Expr ::= ...
       | (let-mode Name strict : Type Expr Expr)
       | (let-mode Name lazy : Type Expr Expr)
       | (let-mode Name memo : Type Expr Expr)
       | (force Name Atom Expr)
```

The `Type` field in `let-mode` is the full mode type. Valid examples:

```text
(strict Int)
(lazy Int {cap fs.read})
(memo (record (a Int) (b String)) {})
(thunk lazy Int {cap fs.read} (raise (cap fs read () Int)))
(let-mode x lazy : (lazy Int {cap fs.read}) (call read-int) (force y x y))
```

Invalid fixture example required by TASK-1661/TASK-1662:

```text
(let-mode x lazy : (memo Int {}) 1 x)
```

Rows use the existing Core row text syntax, including `{}` for closed empty rows.

For Phase 163, `force` is accepted only when `Atom` is `CoreAtom::Var(name)`. The parser may keep
the existing generic `Atom` grammar, but validation/type checking must reject non-variable force
atoms with a structured diagnostic before lowering. This avoids an underspecified checked-type
lookup for arbitrary atoms and can be relaxed in a future phase with an explicit typed-atom table.

### Phase 163 Implementation Addendum

This addendum fixes the runtime/lowering seams that should not be left to task agents. Apply it
when implementing TASK-1663, TASK-1664, TASK-1667, TASK-1669, and TASK-1672.

### CPS Carrier And Force Mechanism

Add these CPS data shapes in `crates/ash-core/src/cps.rs`:

```rust
pub enum ThunkMode {
    Lazy,
    Memo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoCellId(u64);

impl MemoCellId {
    pub fn new(raw: u64) -> Self;
    pub fn raw(self) -> u64;
}

Value::ThunkClosure {
    mode: ThunkMode,
    body: Box<Value>,
    captured_env: Env,
    captured_chain: HandlerChain,
    row: EffectRow,
    memo_cell: Option<MemoCellId>,
}

PrimOp::ForceThunk
```

`body` must be a zero-argument `Value::Lam`; CPS validation rejects any other body value for a
`ThunkClosure`. `memo_cell` is process-local runtime state, defaults to `None` for lazy thunks,
and must not become user-visible program data. Use this exact serialization contract:

- `Value::ThunkClosure.memo_cell` uses `#[serde(skip, default)]`;
- serde round-trips always deserialize `memo_cell` as `None`;
- `.cps` fixture/debug text serializers omit the `memo_cell` field entirely;
- any human diagnostics that must mention memo identity render only `<memo-cell>`, never a
  numeric `MemoCellId`, pointer, or storage address.

Lowering in `ash-core` cannot know the active runtime environment or handler/provider chain. It
must therefore emit `ThunkClosure` values with empty/default `captured_env` and `captured_chain`
placeholders. Runtime construction in `ash-interp` fills those fields when `eval_value` evaluates
the thunk value, mirroring the existing lambda/continuation capture pattern. The thunk body lambda
must not get a separate capture boundary; the thunk carrier's `captured_env` and `captured_chain`
are the authority boundary used at force time.

Runtime capture has one stable construction path. Lowering and fixture serializers emit only
empty/default `captured_env` and `captured_chain` placeholders. `eval_value_with_runtime` overwrites
those empty placeholders exactly once at thunk construction time with the current runtime `Env` and
`HandlerChain`. If a programmatic test constructs a `ThunkClosure` with non-empty capture fields,
the evaluator treats it as already runtime-constructed and preserves those fields. `.core` and
`.cps` fixture text cannot provide live runtime captures.

Add interpreter-owned runtime state in `crates/ash-interp/src/cps/` so `ash-core` does not depend
on interpreter error types:

```rust
pub struct CpsRuntime {
    next_memo_cell: u64,
    memo_cells: HashMap<MemoCellId, MemoCellState>,
    trace: Vec<TraceEvent>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoCellState { Empty, Evaluating, Filled(CachedThunkOutcome) }

#[derive(Debug, Clone, PartialEq)]
pub enum CachedThunkOutcome { Success(Atom), Failure(CpsError) }
```

`TASK-1663` adds only the `trace: Vec<TraceEvent>` sink. It must not add thunk-specific
`TraceEvent` variants or emissions. TASK-1672 owns all thunk/memo trace variants and all emissions.

Add this CPS runtime error variant:

```rust
CpsError::ExpectedThunk(Value)
```

Use `ExpectedThunk` when `PrimOp::ForceThunk` receives an argument that resolves to any value other
than `Value::ThunkClosure`. Do not reuse `InvalidPrimArgs` for this case; `InvalidPrimArgs` remains
for ordinary primitive arity/type failures.

`MemoCellId`'s inner field stays private. `ash-interp` allocates ids with
`MemoCellId::new(raw)`, may read ids with `raw()` only for map keys/debug assertions, and must not
format `raw()` into public diagnostics or traces.

`CpsRuntime::allocate_memo_cell()` returns a fresh `MemoCellId`, inserts `Empty`, and is called
only when a memo `ThunkClosure` with `memo_cell: None` is constructed by runtime `eval_value`.
`eval_value` returns the constructed closure with `memo_cell: Some(id)` before it is bound into the
environment. `ForceThunk` must not allocate a new cell for a memo closure that still has
`memo_cell: None`; that is an invalid runtime state because allocation at force time would not
update all aliases of the closure. Cloned `ThunkClosure` values with the same `MemoCellId` share
the same memo cell within the same `CpsRuntime`. Existing public entrypoints `eval_checked`,
`eval_unchecked`, and `eval_term` create a fresh `CpsRuntime` per top-level call and delegate to
runtime-aware internals, so memo caches do not persist across separate checked runs. Direct runtime
tests may use an explicit `eval_unchecked_with_runtime(term, env, chain, runtime)` helper to
observe cache sharing and traces.

Use these exact runtime-aware helper signatures:

```rust
pub fn eval_unchecked_with_runtime(
    term: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn eval_value_with_runtime(
    value: &Value,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Value>;

fn eval_letprim_with_runtime(
    name: &Name,
    op: &PrimOp,
    args: &[Atom],
    body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn eval_force_thunk_binding(
    name: &Name,
    args: &[Atom],
    continuation_body: &Term,
    env: &Env,
    chain: &HandlerChain,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;

fn run_thunk_body_with_runtime(
    thunk: &Value,
    runtime: &mut CpsRuntime,
) -> CpsResult<Atom>;
```

All recursive evaluator calls must route through the runtime-aware helpers. The old public
entrypoints remain as compatibility wrappers that allocate a fresh runtime and call these helpers.
Do not use thread-local or process-global memo stores in Phase 163.

`Force` lowers to existing `Term::LetPrim { op: PrimOp::ForceThunk, args: [thunk], body }`. The
`ash-interp` CPS evaluator special-cases `PrimOp::ForceThunk` in `eval_letprim` rather than
treating it as an ordinary pure `eval_prim` operation, because it may evaluate the thunk body,
restore the captured handler/provider chain, trap, or replay a cached terminal outcome before
continuing with the `LetPrim.body`.

The force primitive has this exact algorithm:

```text
eval_letprim(name, ForceThunk, args, continuation_body, env, chain, runtime):
  require args.len == 1
  thunk_value = eval_atom_to_value(args[0], env)
  require thunk_value is Value::ThunkClosure, else Err(CpsError::ExpectedThunk(thunk_value))

  if thunk.mode == Lazy:
    outcome = run_thunk_body(thunk, runtime)
    if outcome == Ok(atom):
      # Resume the original force-site body under the original force-site env/chain.
      return eval_unchecked_with_runtime(
        continuation_body,
        env.with_binding(name, Value::Atom(atom)),
        chain,
        runtime
      )
    else:
      return outcome

  if thunk.mode == Memo:
    require thunk.memo_cell is Some(cell)
    # Do not hold a mutable borrow of runtime.memo_cells while evaluating the body.
    state = runtime.memo_cells[cell].clone()
    match state:
      Empty:
        runtime.memo_cells[cell] = Evaluating
        outcome = run_thunk_body(thunk, runtime)
        if outcome is Ok(atom):
          runtime.memo_cells[cell] = Filled(Success(atom))
          # Resume the original force-site body under the original force-site env/chain.
          return eval_unchecked_with_runtime(
            continuation_body,
            env.with_binding(name, Value::Atom(atom)),
            chain,
            runtime
          )
        if outcome is cacheable Err(err):
          runtime.memo_cells[cell] = Filled(Failure(err))
          return Err(err)
        runtime.memo_cells[cell] = Empty
        return outcome
      Evaluating:
        return Err(CpsError::Trap(TrapReason::Custom("re-entrant memo force")))
      Filled(Success(atom)):
        return eval_unchecked_with_runtime(
          continuation_body,
          env.with_binding(name, Value::Atom(atom)),
          chain,
          runtime
        )
      Filled(Failure(err)):
        return Err(err)
```

`run_thunk_body` invokes the zero-argument lambda using a synthetic continuation:

```text
run_thunk_body(thunk, runtime):
  require thunk.body is Value::Lam { params: [], cont, body, ... }
  force_return = Value::Cont {
    param: "__force_result",
    body: Return { value: Var("__force_result") },
    captured_env: Env::new(),
    captured_chain: thunk.captured_chain.clone(),
    consumed: fresh ConsumedFlag,
    row: EffectRow::default(),
  }
  body_env = thunk.captured_env.with_binding(cont, force_return)
  return eval_unchecked_with_runtime(body, body_env, thunk.captured_chain, runtime)
```

The synthetic continuation is internal and must not be serialized into `.cps` fixtures as a
top-level user-visible binding.

Force control flow is therefore fixed:

1. Resolve the force argument to a `Value::ThunkClosure`.
2. Evaluate the thunk body lambda under `thunk.captured_env` and `thunk.captured_chain`.
3. Receive an `Atom` through the synthetic return continuation.
4. Bind that atom as `Value::Atom(atom)` to the `LetPrim.name`.
5. Resume the original force-site `LetPrim.body` under the original force-site `env` and `chain`.

### Memo Terminal Outcome Model

For the current CPS interpreter, cache `CpsResult<Atom>` terminal outcomes:

- cache `Ok(atom)` as `CachedThunkOutcome::Success(atom)`;
- cache `Err(CpsError::Trap(reason))`;
- cache `Err(CpsError::UnhandledEffect(op))` as the lowered recoverable failure/unhandled-effect
  representation until a later phase gives failure a narrower runtime carrier;
- cache other deterministic `CpsError` variants only if they arise from evaluating a well-formed
  thunk body and the task documents the reason in the test name;
- never fill a memo cell for divergence, panic, process termination, or any non-returning runtime
  condition.

Re-entering a memo thunk while its cell is `Evaluating` returns
`Err(CpsError::Trap(TrapReason::Custom("re-entrant memo force".to_string())))`.

### LetMode Lowering Semantics

`LetMode` lowers as follows:

- `LetMode Strict` evaluates the initializer immediately using the existing strict lowering path
  for the initializer expression, then binds the strict result before lowering the body.
- `LetMode Lazy` does not evaluate the initializer at binding time. It wraps the initializer
  expression as a zero-argument thunk body, binds a lazy `ThunkClosure`, and lowers the body with
  the name bound to that thunk value.
- `LetMode Memo` is the same as lazy except the thunk mode is `Memo` and runtime construction
  allocates a memo cell.
- `Force` unwraps `Lazy A` or `Memo A` to the strict inner type `A` and binds that strict value to
  `Force.name` before lowering the force body.

Lazy/memo latent rows are owned by the type checker, not the lowerer. `TASK-1667` computes the
initializer expression row, compares it with the annotated `CoreType::Mode.latent_row`, and rejects
the binding with `CoreTypeCheckError::ModeLatentRowMismatch { name, expected, actual }` when they
disagree. The checker does not mutate the source AST; it returns the checked mode type and records
the computed row in checked-lowering metadata. `TASK-1669` must consume the checked row facts or
explicit `CoreValue::Thunk.row`/`CoreType::Mode.latent_row`; it must not infer latent rows from
lowered CPS terms, and it must not require the initializer expression to already be a
`CoreValue::Thunk`.

Use this exact checked-lowering metadata API:

```rust
pub struct CoreTypeCheckFacts {
    jump_continuation_rows: HashMap<CoreContRef, CoreRow>,
    refinement_obligations: Vec<CoreRefinementObligation>,
    discharges: Vec<CoreContractDischarge>,
    mode_binding_latent_rows: HashMap<CoreName, CoreRow>,
}

impl CoreTypeCheckFacts {
    pub fn mode_binding_latent_rows(&self) -> &HashMap<CoreName, CoreRow>;
}
```

`merge_facts` must extend `mode_binding_latent_rows`, with later facts replacing earlier facts for
the same `CoreName` because normal lexical type checking already prevents ambiguous same-scope
bindings. `lowering_context_with_checked_facts` must copy these rows into a dedicated lowering
context table named `mode_binding_latent_rows`. `CoreExpr::Force` lowering reads that table by the
forced thunk binding name when the thunk atom is `CoreAtom::Var(name)`. Lowering must reject any
remaining non-variable force atom as an internal checked-lowering invariant violation, because
validation/type checking should already have rejected it.

Use these exact lowering-context helpers:

```rust
impl CoreLoweringContext {
    pub fn with_mode_binding_latent_row(self, name: CoreName, row: CoreRow) -> Self;
    pub fn mode_binding_latent_row(&self, name: &str) -> Option<&CoreRow>;
}
```

### Re-Entrant Memo Test Shape

TASK-1664 must include a direct CPS runtime test that constructs a `Memo` `ThunkClosure`, inserts it
into its captured environment under its own name, and uses a zero-argument body that forces that same
name through `PrimOp::ForceThunk`. The first force sets the cell to `Evaluating`; the nested force
observes `Evaluating` and traps. TASK-1671 may add a Core fixture using `LetRec` only if the Core
syntax can express the same self-reference without broadening the phase.

### Conversion Operations

Do not add new named Core builtins for `delay`, `delay_memo`, `force_unsafe`,
`memoize_unsafe`, or `strip_cache_unsafe` in Phase 163. TASK-1669 must document the Core
translations instead:

- `delay(v)` is represented as a lazy `CoreValue::Thunk` whose body returns `v`;
- `delay_memo(v)` is represented as a memo `CoreValue::Thunk` whose body returns `v`;
- `force_unsafe(t)` is represented as `CoreExpr::Force`;
- `memoize_unsafe(lazy_t)` is represented as a memo thunk whose body forces `lazy_t`;
- `strip_cache_unsafe(memo_t)` is represented as a lazy thunk whose body forces `memo_t`.

If a future surface/elaboration phase introduces these names as functions, it must lower them to
these Core forms before Phase 163 lowering runs.

### Tracing API Location

TASK-1672 must add thunk trace events to the existing execution trace surface:
`ash_core::TraceEvent` in `crates/ash-core/src/provenance.rs`, with emission helpers in
`crates/ash-interp/src/execution_record.rs` or the CPS interpreter module that owns the final
force implementation. Tests observe the public `ExecutionRecord::trace()` /
`SemanticWorkflowOutcome::trace()` path where available; direct CPS-only tests may use a small
runtime trace sink added alongside `crates/ash-interp/src/cps/mod.rs` if the CPS evaluator is still
separate from the semantic execution-record pipeline.

Use these exact public trace event variants. `MemoCellId` and raw storage addresses must not appear
in any payload. The CPS runtime trace sink is `Vec<TraceEvent>`, not a separate internal event type;
direct CPS tests and semantic execution-record tests inspect the same event shape.

```rust
TraceEvent::ThunkConstructed { mode: String, row: Vec<String>, timestamp: DateTime<Utc> }
TraceEvent::ThunkForceStarted { mode: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkBodyEvaluationStarted { mode: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkBodyEvaluationCompleted { mode: String, outcome: String, timestamp: DateTime<Utc> }
TraceEvent::ThunkForceCompleted { mode: String, outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoCacheFilled { outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoCacheHit { outcome: String, timestamp: DateTime<Utc> }
TraceEvent::MemoReplayFailure { reason: String, timestamp: DateTime<Utc> }
TraceEvent::MemoReentrantRejected { timestamp: DateTime<Utc> }
```

Update `trace_event_timestamp` in `crates/ash-interp/src/execution_record.rs` so these new variants
participate in existing trace sorting/projection behavior.

Use these exact outcome strings in trace payloads:

| Runtime outcome | Trace string |
|-----------------|--------------|
| `Ok(_)` | `"success"` |
| `Err(CpsError::Trap(_))` | `"trap"` |
| `Err(CpsError::UnhandledEffect(_))` | `"unhandled-effect"` |
| any other `Err(CpsError::...)` | `"runtime-error"` |

`MemoReplayFailure.reason` uses the same string mapping. Do not introduce task-local synonyms such
as `"failed"`, `"error"`, or `"panic"` in Phase 163 tests.

Fixture/runtime tests observe repeated execution through trace counts:

- lazy re-run: two `ThunkBodyEvaluationStarted { mode: "lazy" }` events after forcing the same
  thunk twice;
- memo single-run: one `ThunkBodyEvaluationStarted { mode: "memo" }`, one `MemoCacheFilled`, and
  one `MemoCacheHit` after forcing the same thunk twice;
- cached failure/trap replay: one body-evaluation event, one cache-fill failure event, and one
  `MemoReplayFailure` on the second force;
- captured authority: assert the thunk-body raised operation is handled by the creation-time
  chain, then confirm the trace has one body-evaluation event under the forced thunk.

## Task Table

| Task | Description | Est. Hours | Dependencies | Status |
|------|-------------|-----------:|--------------|--------|
| [TASK-1660](tasks/TASK-1660-core-mode-ast-carriers.md) | Add Core mode type, thunk value, LetMode, and Force AST carriers | 3 | Phase 162 | Done |
| [TASK-1661](tasks/TASK-1661-core-mode-text-format.md) | Parse, serialize, and round-trip `.core` mode syntax and fixtures | 4 | TASK-1660 | Done |
| [TASK-1662](tasks/TASK-1662-core-mode-validation.md) | Validate mode/type agreement, thunk shape, Force shape, and binder scoping | 3 | TASK-1661 | Done |
| [TASK-1663](tasks/TASK-1663-cps-thunk-carrier.md) | Add CPS ThunkClosure value carrier and memo-cell state scaffolding | 4 | TASK-1660 | Done |
| [TASK-1664](tasks/TASK-1664-cps-force-runtime.md) | Implement CPS lazy/memo force runtime behavior, cached outcomes, and re-entrant rejection | 5 | TASK-1663 | Done |
| [TASK-1665](tasks/TASK-1665-core-mode-type-wellformedness.md) | Type-check mode type well-formedness and mode invariance diagnostics | 3 | TASK-1662, TASK-1641 | Done |
| [TASK-1666](tasks/TASK-1666-core-thunk-value-typing.md) | Type thunk values with latent rows and pure construction row | 4 | TASK-1665, TASK-1643 | Done |
| [TASK-1667](tasks/TASK-1667-core-letmode-force-typechecking.md) | Type LetMode and Force expressions with SPEC-101 row accounting | 5 | TASK-1666, TASK-1644 | Done |
| [TASK-1668](tasks/TASK-1668-core-mode-public-summaries.md) | Preserve mode and latent-row facts in public summaries and diagnostics | 3 | TASK-1667, TASK-1649 | Done |
| [TASK-1669](tasks/TASK-1669-core-mode-lowering.md) | Lower thunk construction, strict/lazy/memo LetMode, and Force into CPS thunk runtime forms | 5 | TASK-1664, TASK-1667 | Done |
| [TASK-1670](tasks/TASK-1670-core-thunk-capture-authority.md) | Verify captured handler/provider-chain authority at force time | 4 | TASK-1669 | Done |
| [TASK-1672](tasks/TASK-1672-core-mode-tracing-observability.md) | Add thunk construction/force/memo trace events and observability tests | 4 | TASK-1664, TASK-1669 | Done |
| [TASK-1671](tasks/TASK-1671-core-mode-end-to-end-fixtures.md) | Add parse -> validate -> type-check -> lower -> run fixtures and golden examples | 5 | TASK-1670, TASK-1672 | Done |
| [TASK-1673](tasks/TASK-1673-core-lazy-memo-reference-closeout.md) | Document behavior, reconcile tracking, and close out Phase 163 | 3 | TASK-1671 | Done |
| [TASK-1674](tasks/TASK-1674-core-force-function-row-remediation.md) | Preserve forced function rows and scoped LetMode bindings during checked lowering | 2 | TASK-1667, TASK-1669 | Done |

**Total estimated hours:** 57.

## TDD Policy

Every task must remain small and TDD-oriented:

1. Add one or two focused failing tests named for the behavior.
2. Run the exact focused test and record the expected failure in the task notes.
3. Implement the minimum code for that slice.
4. Run the focused test until it passes.
5. Run the affected crate gate.
6. Update the task file completion evidence, PLAN-INDEX status, and CHANGELOG.
7. Commit before starting the next task.

## Required Examples and Fixtures

The phase must include at least these example programs before closeout:

- `lazy_reruns.core`: repeated force of a lazy thunk re-runs the body and charges the latent row at each force site.
- `memo_runs_once.core`: repeated force of a memo thunk reuses the cached terminal outcome.
- `memo_caches_failure.core`: memo force caches recoverable failure or trap replay without re-running preceding effects.
- `memo_reentrant_trap.core`: re-entrant memo force deterministically traps.
- `force_captured_handler.core`: an effectful thunk dispatches through the handler/provider chain captured at thunk construction.
- `mode_mismatch_invalid.core`: malformed `LetMode.mode`/`LetMode.ty` mismatch is rejected.

## SPEC-101 Coverage Matrix

| SPEC-101 requirement | Required task coverage | Required test/fixture evidence |
|----------------------|------------------------|--------------------------------|
| Distinct `Strict`/`Lazy`/`Memo` types | TASK-1660, TASK-1665 | `task_1660_core_mode_ast`, `task_1665_core_mode_type_wellformedness` |
| Mode mismatch rejection | TASK-1662, TASK-1665, TASK-1667 | `mode_mismatch_invalid.core`, validator/typechecker mismatch tests |
| Lazy force re-runs body | TASK-1664, TASK-1671 | `lazy_reruns.core`, `task_1664_cps_force_runtime` |
| Memo force runs body once | TASK-1664, TASK-1671 | `memo_runs_once.core`, memo cache-hit runtime test |
| Memo caches failures/traps | TASK-1664, TASK-1671 | `memo_caches_failure.core`, cached trap replay runtime test |
| Re-entrant memo force rejection | TASK-1664, TASK-1671 | `memo_reentrant_trap.core`, structured trap assertion |
| Thunk construction row `{}` | TASK-1666, TASK-1667 | thunk value and `LetMode` row-accounting tests |
| Force contributes latent row | TASK-1667, TASK-1669 | force row typechecker test and lowered row assertion |
| Captured handler/provider chain | TASK-1669, TASK-1670 | `force_captured_handler.core`, lowering/runtime authority test |
| Value-level thunk carrier lowering | TASK-1663, TASK-1669 | lowered CPS `ThunkClosure` structural assertions |
| Runtime trace events | TASK-1672 | trace tests for construction, force, fill, hit, replay, and rejection |

## Verification Gates

### Focused per-task gates

```bash
cargo test -p ash-core --test task_1660_core_mode_ast
cargo test -p ash-core --test task_1661_core_mode_text
cargo test -p ash-core --test task_1662_core_mode_validation
cargo test -p ash-core --test task_1665_core_mode_type_wellformedness
cargo test -p ash-core --test task_1666_core_thunk_value_typing
cargo test -p ash-core --test task_1667_core_letmode_force_typecheck
cargo test -p ash-core --test task_1668_core_mode_public_summary
cargo test -p ash-core --test task_1669_core_mode_lowering
cargo test -p ash-core --test task_1670_core_thunk_capture_authority
cargo test -p ash-core --test task_1672_core_mode_tracing_docs_consistency
cargo test -p ash-core --test task_1671_core_mode_end_to_end
cargo test -p ash-interp --test task_1663_cps_runtime_scaffold
cargo test -p ash-interp --test task_1664_cps_force_runtime
```

### Affected-crate gates

```bash
cargo test -p ash-core
cargo test -p ash-interp
cargo clippy -p ash-core --all-targets -- -D warnings
cargo clippy -p ash-interp --all-targets -- -D warnings
cargo fmt --check
git diff --check
```

### Documentation gate

```bash
cargo test -p spec_processor spec_links
```

## Acceptance Criteria

- [ ] Core mode AST carriers exist for `Strict`, `Lazy`, `Memo`, thunk values, `LetMode`, and `Force`.
- [ ] `.core` parser and serializer round-trip representative mode forms.
- [ ] Validator rejects malformed mode/type disagreement and invalid force/thunk shapes.
- [ ] CPS value layer has a thunk carrier with captured environment and handler/provider chain.
- [ ] CPS runtime implements lazy re-run, memo single-run, cached terminal outcomes, and re-entrant memo rejection.
- [ ] Type checker treats mode types invariantly and validates inner types.
- [ ] Thunk construction has local row `{}` while preserving latent row metadata.
- [ ] `Force` contributes the thunk latent row and returns the strict inner type.
- [ ] Public summaries preserve mode type and latent-row information.
- [ ] Core-to-CPS lowering uses thunk carriers and runtime force, not plain calls for effectful thunks.
- [ ] Lowering tests prove captured authority is creation-time, not force-time.
- [ ] End-to-end fixtures cover lazy re-run, memo cache hit/fill, cached failure/trap, re-entrant rejection, row accounting, and mode mismatch.
- [ ] Runtime traces distinguish construction, force, cache fill, cache hit, replay, and re-entrant rejection without exposing raw memo-cell addresses.
- [x] Reference docs, PLAN-INDEX, task files, and CHANGELOG are reconciled.

## Recommended Execution Order

```text
TASK-1660 -> TASK-1661 -> TASK-1662
      |                      |
      v                      v
TASK-1663 -> TASK-1664   TASK-1665 -> TASK-1666 -> TASK-1667 -> TASK-1668
      |                                                |
      v                                                v
TASK-1669 -> TASK-1670 -> TASK-1672 -> TASK-1671 -> TASK-1673
TASK-1674 (review remediation after TASK-1673)
```

## References

- [SPEC-101: Lazy and Memo Computation Modes](../spec/SPEC-101-LAZY-AND-MEMO-COMPUTATION-MODES.md)
- [SPEC-100: Core Type Checking](../spec/SPEC-100-CORE-TYPE-CHECKING.md)
- [SPEC-099: Core Language](../spec/SPEC-099-CORE-LANGUAGE.md)
- [SPEC-098b: Target IR](../spec/SPEC-098b-TARGET-IR.md)
- [SPEC-097b: Target Type System](../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-096b: Target Effect System](../spec/SPEC-096b-TARGET-EFFECT-SYSTEM.md)
- [PLAN-161: Core Ash IR Foundation](PLAN-161-CORE-ASH-IR-FOUNDATION.md)
- [PLAN-162: Core Ash Type Checking](PLAN-162-CORE-ASH-TYPE-CHECKING.md)

## Changelog

- 2026-06-21: Created Phase 163 plan for implementing SPEC-101 lazy and memo computation modes in Core Ash with self-contained TDD tasks, required examples, lowering tests, runtime tests, and closeout criteria.
