# TASK-2013: Source Handlers and `handle ... with` Lowering

**Status:** In progress — canonical parser/AST carriers, a checked-handler declaration sidecar,
and a nonproduction typed Core `Handle`/`Raise` inspection bridge are implemented. Its closed-empty
identity subset preserves `MultiShotPure` resume multiplicity through Core validation, typechecking,
and CPS inspection; the single direct `resume(arg)` form retains declared-payload unification.
The Engine can retain entry-owned checked handler facts only for the exact checked
entry anchor, and a TASK-2014 V1 seam can project checked handler/application
facts. One separately sealed TASK-2014 production token now consumes the exact checked
`absorb_sleep` fixture through `Engine::run` and `run_file`; it is not generic handler execution.
General residual-row semantics, validated source-handler admission beyond that fixture, and runtime
integration remain unimplemented. All other public Engine routes remain closed, while the
handler-free and constructor controls continue to reject CPS `Raise`/`Handle` terms.
**Phase:** Follow-up from [TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md),
[TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md), and
[TASK-2012](TASK-2012-declared-operation-provider-binding.md)

**Status:** In progress

**Semantic task record:** [TASK-2013 workflow record](../semantic-task-records.json)

**Semantic coverage map:** [TASK-2013 semantic workflow record](../SEMANTIC-RULE-COVERAGE.md#task-2013-semantic-workflow-record)

**Implementation:** partial
**Evidence:** tested
**Parity:** below_spec
**Missing target-spec clauses:** Connect validated typed handler lowering to separately authorized production admission.

## Semantic workflow record

Checked handler clauses and closed inspection facts do not realize the complete target handler
rules. The tests listed in the record provide confidence only for the realized behavior.

## Description

Implement the canonical source handler surface through typed Core lowering: `on comp { ... }`
operation clauses, `done` clauses, handler-marked callable registration, and
`handle expr with handler_name` sugar.  A source handler must use concrete impl-qualified
operation identity, type its resume continuation against the residual row, and lower to exact Core
`Handle`/`Raise` forms with source anchors and local residual rows.

This task owns source AST, parser, registration/typechecking, and source-to-Core lowering only.
Engine runtime integration is required by TASK-2014 Path B only after those source/lowering
artifacts are validated into its explicit admission artifact. This task supplies checked clause,
operation, residual-row, and source-anchor facts; TASK-2014 separately admits provider bindings
and authorizes any frame installation. It is not an implied outcome of parser acceptance, a
handler marker, or a row.

TASK-2001's V8 imported-row summaries are checked structural input only: they may normalize a
declared operation identity, evidence/tail, or another currently parseable row requirement while
retaining imported-use provenance. A legacy V7 text-only provider/binding row remains decodable
only for compatibility and rejects before typed-handler normalization with the required V8-content
diagnostic. Neither summary version selects a provider, installs a frame, or supplies admission
authority.

## Authoritative References

- [SPEC-095b §4.3](../../spec/SPEC-095b-TARGET-GRAMMAR.md#43-handler-expressions): canonical `on` clauses and `handle expr with identifier`; historical inline `handle effect_item with { ... }` is removed.
- [SPEC-095b §6.4](../../spec/SPEC-095b-TARGET-GRAMMAR.md): handler declaration marker and one-way `handler fn <: fn` coercion.
- [SPEC-097b §8.8](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md#88-handler-typing): thunk input, residual rows, answer type, and affine/multi-shot continuation discipline.
- [SPEC-098c §6](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md#6-handlers-and-provider-boundaries): handler installation/lowering and provider-boundary non-synthesis.
- [SPEC-099](../../spec/SPEC-099-CORE-LANGUAGE.md): Core handler clause and local `Handle.row` contract.
- [SPEC-098b §5 and §10](../../spec/SPEC-098b-TARGET-IR.md): operation-typed `Raise`/`Handle`, local residual rows, and handler/provider dispatch shape.
- [SPEC-099b](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md): frame ordering and the rule that rows never install frames.
- [TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md#parserast-handler-and-newtype-slice): existing handler declarations are structural/registered metadata only; canonical clauses and lowering remain open.

## Scope

### In scope

- Parser and AST carriers for the exact canonical forms:

  ```ash
  on computation {
      ImplType::operation(pattern, resume) => body,
      done(value) => done_body,
  }

  handle expression with handler_name
  ```

- Handler declaration/type registration that preserves the existing handler marker, distinct value
  namespace identity, source spans/origins, callable signature, and marker-only admission rule.
- Typechecking concrete impl-qualified clause identity against the same declaration-backed
  operation resolver used by TASK-2011; validation of clause payload/result types, `done` result,
  common answer type, and residual row.
- Explicit resume continuation typing `OpResult -> {residual} Answer`, with the existing
  multiplicity machinery enforcing affine continuation use for non-empty residual rows and
  multi-shot behavior only for empty residual rows.
- Desugaring `handle expr with h` only after normal value-name resolution proves that `h` is
  handler-marked; a plain compatible `fn` must reject.
- Exact typed Core lowering to `CoreExpr::Handle` / `CoreExpr::Raise` (and then existing checked
  Core-to-CPS machinery), preserving concrete operation identity, clause/done/resume binders,
  local residual row, and source origins.

### Explicit deferrals

- Historical `handle effect_item with { ... }` syntax and all workflow/tower compatibility forms.
- TASK-2001 materializes the `derive handler` source/typechecker fact for every direct
  impl method, including multiple-operation synthesis, open residual/answer facts, affine
  continuations, and derive-site provenance. The local source application route now resolves the
  derived name through the normal handler marker plus that checked fact, instantiates its answer
  from the operand result and its residual from the actual normalized operand row, and retains
  operand/handler anchors. Its coverage of currently parseable aliases/groups/open tails is only
  through the narrow explicit zero-argument call of a row-annotated parameter; unsupported
  computations still reject. See
  [TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md). Lowering that fact to
  Core/CPS, imported or cross-module handler resolution, and handler re-export behavior beyond
  existing metadata remain deferred.
- Generic/interface-qualified/binding-name operation clauses, monomorphization, and general
  overlap/coherence selection.
- General Engine installation/execution of source handlers, provider-frame construction, and
  production artifact consumption. TASK-2014 Path B now admits only one exact closed-empty local
  `absorb_sleep` handler over `TestClock::sleep(Int) -> Int`: its direct `resume(ms)`, identity
  `done`, and literal `0` result are checked/lowered to one root `SourceHandler` instruction and
  executed by `Engine::run`/`run_file` using a separate opaque same-Engine token. That token
  authorizes exactly one engine-private checked-CPS handler installation/dispatch; it selects no
  provider, constructs no provider frame, and derives no authority from a row. Every other
  handler-bearing source body rejects at admission; the handler-free positive and
  constructor tokens also reject nested `Raise`/`Handle`. Production frame construction, async
  host-operation driving, and CLI handler-route integration remain implementation work.
- TASK-2005 separately compares that same source shape only as one manifest-fingerprinted private
  differential tuple. Its opaque checked-handler inspection terminalization is not this task's
  source-to-Core lowering, a production token, `Engine::run`, or authority to install a frame.
- Dynamic contracts as resumable handler clauses, provider inference from rows, direct `invoke`,
  and any `Act<T>`/`Proc<T>` restoration.

## Target APIs and Data Flow

1. **Parser/surface:** extend `crates/ash-parser` expression and declaration carriers rather than
   encoding clauses in raw strings or reusing removed syntax.  Preserve spans for `on`, each
   `ImplType::operation`, pattern, resume binder, `done`, and `handle ... with` handler name.
2. **Registration/type environment:** extend the existing `Definition::Handler(HandlerDef)`,
   `CallableDeclarationKind::Handler`, and handler-only lookup path in `ash-typeck::TypeEnv`.
   Add a typed clause/handler signature carrier, not a provider or runtime frame.
3. **Expression checking:** derive an immutable `CheckedComputation` from the existing source AST
   before either canonical `on` checking or `handle ... with` admission.  Declared concrete
   operation calls contribute anchored singleton rows; audited pure composites structurally union
   inferred child rows; row-bearing annotations/signatures use the structural row normalizer.
   Unclassified expression forms fail closed rather than becoming implicitly pure. Use
   `TypeEnv::resolve_declared_concrete_operation` for every concrete clause. Compute the body row,
   subtract only clauses actually handled, validate answer type and continuation
   type/multiplicity, and reject an ordinary function in `handle ... with`.
4. **Core lowering:** add source-to-Core lowering that constructs existing Core `Handle` and
   `Raise` carriers from typed handler facts.  `Handle.row` is the local residual body row; it is
   not the total continuation row and it never grants provider/handler authority.
5. **CPS inspection:** use existing Core-to-CPS validation/lowering only to inspect the resulting
   handler/raise shape.  Do not call it from production execution in this task.

## Requirements

1. Parser acceptance is exact and fail-closed.  Require one or more concrete impl-qualified
   operation clauses and exactly one `done` clause in the selected initial grammar; reject missing
   `done`, duplicate `done`, interface-qualified clauses, malformed continuation binders, and the
   removed inline historical form with deterministic diagnostics.
2. A handler declaration registers a marker-bearing callable type.  It can coerce to a plain
   function only where ordinary function use is accepted; the reverse coercion is forbidden, so
   `handle expr with ordinary_fn` rejects even if its apparent function shape matches.
3. For `on`, each clause operation resolves through declared concrete impl metadata, its payload
   matches the interface-declared signature, and its resume binder receives exactly
   `OpResult -> {residual} Answer`.  Clause and `done` bodies must agree on `Answer`.
4. Residual rows are exact: handled concrete items are peeled once from the computation row,
   handler-body effects are included as specified, and the resulting local Core `Handle.row`
   excludes the outer continuation row.  Rows never install handler/provider frames.
5. `handle expr with h` first implicitly thunks and infers its source expression as immutable
   `Unit -> {R} A` evidence, then resolves `h` in the value namespace, verifies its handler
   marker and exact normalized thunk input, preserves handler/expression/row origins, and lowers
   to the same canonical handler installation artifact as the corresponding explicit typed handler
   application. Inference is finite and fail-closed: unsupported AST forms are errors, never
   silently pure computations.
6. Core lowering emits exact `Handle`/`Raise` operation identities, declared argument/result types,
   resume/done binders, residual rows, and source anchors.  It must not lower the source operation
   as ordinary `Call`/`FnApply` or synthesize a provider frame.
7. TASK-2014 Path B makes validated typed lowering a production-admission prerequisite. This task
   must provide its source facts to the explicit admission artifact; existing TASK-1993 frame-order
   evidence is preserved but does not itself make a source handler executable or authorize a frame.

## TDD Steps

1. **RED: surface grammar.** Add parser fixtures for a single local concrete declared operation,
   one `on` clause plus `done`, a handler declaration, and `handle expr with handler_name`.
   Add rejection fixtures for historical inline syntax, missing/duplicate `done`, malformed
   `ImplType::op(pattern, resume)`, and interface-qualified clauses.
2. **GREEN: AST only.** Implement source carriers/spans and parser support.  Verify no parser
   route creates provider/handler runtime authority.
3. **RED: computation inference, marker and typing.** Add typechecker tests for immutable
   AST-directed computation facts: declared concrete operation calls, audited pure composition,
   alias/group/open-tail/non-operation source rows, source anchors, exact row union, and
   fail-closed unsupported forms. Add a handler-marked callable accepted by implicit
   `handle expr with`, an ordinary function rejected there, declared-clause identity/signature
   mismatch, answer-type mismatch, and residual-row/multiplicity cases.
4. **GREEN: inference, registration and row typing.** Implement fail-closed computation
   inference, marker-aware handler callable registration, clause resolver use, answer/residual
   typing, and continuation multiplicity checks.
5. **RED: exact Core.** Add source-to-Core tests asserting `Handle` and nested/contained `Raise`
   identity, declared signature, resume and done binders, local residual row, and source anchors.
   Assert a matching operation row alone does not create a provider frame.
6. **GREEN: lowering.** Implement the smallest typed lowering bridge.  Feed it to private checked
   Core/CPS inspection; assert production execution does not invoke that bridge.
7. **Regression gates.** Keep TASK-2000 `invoke` rejection, TASK-2011/2012 concrete operation and
   binding tests, and TASK-1993 frame-order tests passing.  Run affected parser/typechecker/core/
   engine/interpreter tests, format, Clippy, docs/traceability gates, and `git diff --check`.

## Completion Checklist

- [x] Canonical `on` and `handle ... with` parse with structural source origins; existing removed-form rejection remains separate Phase-201/TASK-2001 evidence.
- [x] Handler marker registration is preserved and plain functions reject in handler position.
- [x] Clauses resolve only concrete declared operation identities with exact declared signatures.
- [ ] General resume continuation, common answer type, residual row, and multiplicity behavior are checked; the direct one-argument affine resume form is checked.
- [x] A deliberately narrow closed-empty identity inspection bridge yields exact Core
  `Handle`/`Raise` artifacts, preserves `MultiShotPure` resume multiplicity through Core/CPS
  inspection, and has no provider synthesis; general lowering remains open.
- [ ] Existing direct `invoke` rejection, row non-granting, and handler/provider frame-order evidence remain intact.
- [x] Engine-retained checked handler facts are constrained to the same Engine and exact checked
  entry anchor. One closed-empty exact `absorb_sleep` fixture has canonical parsed
  source/Core provenance, one checked root `SourceHandler` instruction, and a separate same-Engine
  opaque production token consumed only by `run`/`run_file`; all other handler source forms remain
  closed. It has one authorized checked-CPS handler installation/dispatch, but no provider binding,
  provider frame, row-derived authority, or general/multi-frame installation.
- [ ] Tests, formatting, Clippy, changelog, plan/index, traceability, docs gate, and diff checks pass.

## Completed Checked-Handler Sidecar Stage

`type_check_program` now traverses each parsed `handler` declaration through a dedicated
`check_handler_declarations` pass before ordinary entry checking. It registers and retains a
`CheckedHandlerDeclaration` keyed by handler name, with the existing `CallableDeclarationKind::Handler`
marker, checked callable signature, and `CheckedHandlerClause` facts for later lowering. This is
checked declaration metadata only: it neither installs a frame nor reaches the engine/runtime.

Each operation clause resolves through `TypeEnv::resolve_declared_concrete_operation`, preserving
the exact concrete impl identity, interface, operation, declared parameter/result types, payload
type, and resume binder name. Clause bodies and the `done` body are checked against the handler
answer type; duplicate or missing `done` clauses reject. An unknown clause operation fails during
this traversal with the declaration-backed diagnostic (for example,
`concrete impl 'TestClock' has no operation 'wake'`), before lowering.

The ordinary expression checker treats `on` as unavailable outside this declaration validation
pass. The sidecar's `handle expr with handler_name` check requires the existing
handler-only callable marker and unifies the handler input with the checked handled-expression
type; an ordinary compatible function rejects as an ordinary function rather than a handler. The
approved row-aware follow-up supersedes that input premise with one explicit
AST-directed, fail-closed `CheckedComputation` inference stage: it supplies the implicit
`Unit -> {row} result` evidence for both `on expr` and `handle expr with h`, without creating a
runtime thunk. The sole continuation form in this stage is a whole-clause
`resume(arg)`: `arg` unifies with the declared operation result, the clause
gets the handler answer type, and its affine continuation may occur once. An all-resume block with
more than one direct invocation receives the affine duplicate diagnostic. Nested, malformed,
zero-argument, and extra-argument calls do not acquire continuation semantics and instead remain
ordinary checking failures.

This sidecar itself performs no residual-row subtraction, general continuation multiplicity,
provider or handler frame creation, source-handler admission, direct execution, or production CPS
invocation. The later nonproduction inspection bridge consumes only its checked facts; it does not
broaden these sidecar semantics. The focused evidence is
[`task_2013_checked_handler_declaration.rs`](../../../crates/ash-typeck/tests/task_2013_checked_handler_declaration.rs).

## Completed Typed Core Handler Raise Inspection Stage

`lower_checked_handler_application_to_core` is a test-driven, nonproduction lowering boundary from a
checked source handler application into existing `CoreExpr::Handle` and `CoreExpr::Raise` carriers.
Its focused evidence fixture is the locally declared `TestClock::sleep(0)` operation under an
`echo_sleep` handler. That fixture reconstructs the exact concrete operation identity and its declared
`Int -> Int` signature, emits a literal `0` `Raise` beneath a `Handle`, and gives the handler
clause an empty local residual row. The artifact is then passed through existing Core validation,
Core type checking, and Core-to-CPS lowering solely for structural inspection; the inspected CPS
term is a `Handle` containing the exact capability `Raise` and an empty handled row.

The bridge admits only one concrete operation clause, a closed-empty output row, an identity
operation-clause body over one variable payload binder, an identity `done` clause, and literal
operation arguments. It preserves the source continuation multiplicity: the closed-empty
`echo_sleep` fixture carries `Cont<Int, Int, {}, MultiShotPure>` through Core validation, checking,
and CPS lowering. The exact direct one-argument `resume(ms)` body lowers to
`CoreExpr::Jump { cont: Var(resume), arg: Var(ms) }`; the argument must unify with the declared
operation result. Nested, malformed, zero-argument, or extra-argument forms remain ordinary-
checking failures. A nonidentity `done` clause rejects because the current Core `Handle` carrier
has no return-clause representation. Nonempty/open output rows and multiple operation clauses
remain outside this slice. These restrictions are intentional: they make no claim about general
continuation invocation, answer transformation, residual-row subtraction, or handler runtime.

By itself this bridge creates no provider/handler frame and is not a production runtime path.
TASK-2014 consumes only its closed-empty identity `echo_sleep` result through an opaque,
Engine-issued V1 wrapper with one exact root source-handler instruction and exact answer
terminalization. That execution adds no ordered frame installation/TASK-1993 operational
dispatch, provider/residual handling, async timeout/cancellation, or production routing. The
focused evidence is
[`task_2013_handler_core_lowering.rs`](../../../crates/ash-typeck/tests/task_2013_handler_core_lowering.rs).

TASK-2024 adds one separate, equally private non-empty local-row control: a `forward_sleep`
handler handles `TestClock::sleep(0)` and its clause raises `TestClock::wake(ms)`.  It produces
exact `Raise(sleep, Int(0))` / `Raise(wake, Var(ms))` Core and CPS carriers with
`Handle.row = {TestClock::wake}`.  The two negative controls (`other(ms)` and `wake(0)`) reject
before an inspection artifact exists.  This is evidence only for that exact local row; it does
not extend TASK-2013's continuation, runtime, provider, admission, or production boundary.

TASK-2026 is a separate Engine-owned consumer of that exact retained fixture. Its completed
production route seals source/Core/anchor provenance, the checked handler facts, concrete
`sleep`/`wake` identities, one exact `wake` provider binding, and explicitly authorized outer
Provider then inner SourceHandler instructions. It proves normal return and the canonical
timeout/cancellation envelope for that one `wake` await without widening this typed lowering
bridge: rows alone still never install frames, and all general handler, continuation,
residual-row, CLI, trace, and generic-execution behavior remains closed.

## Completed Parser/AST and Fail-Closed Lowering Stage

`crates/ash-parser` now represents `on` as `Expr::On` with `HandlerClause::Operation` and
`HandlerClause::Done` variants. The operation clause preserves its concrete impl-type spelling,
operation name, payload pattern, resume binder, body, and clause span; the `done` clause preserves
its completion binder, body, and span. The enclosing `on` span and parsed module source path are
retained. `handle expression with handler_name` is separately represented as `Expr::HandleWith`,
preserving both the nested expression and value-namespace handler name with its full span.

The parser now requires at least one operation clause and exactly one `done` clause, rejecting
missing or duplicate forms deterministically with source-oriented diagnostics. It does not yet
validate declared/concrete clause identity, handler markers or callable signatures,
payload/resume/answer/residual rows, or continuation multiplicity; nor does it establish Core
lowering or runtime behavior. Those invariants require the typed stage rather than parse-time
heuristics.

`lower_expr` rejects both new expression variants with the stable fail-closed message
`source handlers require typed handler lowering before Core lowering`. Therefore no ordinary Core
call, synthetic provider frame, direct-runtime installation, or execution is produced from the new
forms. Existing macro/notation traversal visits the computation and clause bodies while preserving
the structural source carriers; it does not establish handler semantics.

The focused parser evidence is
[`task_2013_handler_surface.rs`](../../../crates/ash-parser/tests/task_2013_handler_surface.rs),
which proves operation identity/binders/bodies/origins and `handle ... with` handler-reference/span
preservation. The checked-handler sidecar adds marker admission and declaration facts, but Core
`Handle`/`Raise`, residual rows, resume-call typing, handler runtime frames, and production
execution remain open work.

## Completed Canonical `on` Cardinality Stage

The approved structural slice now enforces the canonical `on` shape at both admission
boundaries: source parsing requires at least one operation clause and exactly one `done` clause,
and `check_handler_declarations` independently applies the same guard to a constructed AST before
it can publish checked-handler facts.  The latter rejects zero-operation, missing-`done`, and
duplicate-`done` constructed forms; source text is rejected first by the parser.  Missing forms
have stable, source-oriented diagnostics at the enclosing `on` expression, while a duplicate
`done` deterministically identifies the second `done` source position.

This is cardinality only.  It neither adds a duplicate-concrete-operation rule nor changes the
symbolic `ImplType::operation` carrier, direct `Expr::On` checking boundary, handler typing,
residual rows, Core/CPS lowering, provider or handler frames, or runtime execution.  The task
therefore remains **In progress** and its unchecked completion criteria and explicit deferrals
remain in force.

## Completed `on expr` Stop-Set Parser Stage

The approved parser-only extension now parses the computation following `on` through the existing
expression grammar.  Focused structural evidence covers call (`on run(req) { ... }`), binary
(`on retries + 1 { ... }`), record-literal (`on { request: run(req) } { ... }`), and named record
constructor (`on Result { value: run(req) } { ... }`) computations.  In every case `Expr::On`
retains both its outer source span/origin and the computation's own structural carrier and span.

The parser-local `on` computation mode stops only at a *top-level*, clause-shaped, non-consuming
`{` opener (`done(` or `identifier::identifier(` after trivia).  The first brace of a record
literal or named constructor remains part of the computation; the subsequent clause brace starts
the handler.  Terminal line comments (`//`, `--`) and nested block-comment trivia are trimmed for
that lookahead, while internal comments and quoted marker text remain expression content rather
than delimiters.  Clause parsing and the existing cardinality rules therefore begin at their
original boundary and retain their diagnostics.

This stage changes no handler semantics: cardinality remains unchanged, and it adds no typing,
residual-row, continuation, Core/CPS, provider/frame, or runtime behavior.  TASK-2013 remains
**In progress** under its existing unchecked criteria and explicit deferrals.

Verified evidence: TASK-2013 parser surface 14/14, checked-handler declaration 9/9, Core lowering
15/15, the full parser suite, `cargo fmt --check`, parser Clippy with warnings denied,
`git diff --check`, and the documentation gates.

## Current Row-Aware Source Typing Evidence

TASK-2013 remains **In progress**.  The source typechecker now records immutable
`CheckedComputation`, checked-handler, and checked-application facts only: exact normalized input
rows, residual rows, shared answer types, clause/done output effects, continuation rows and
affine/multi-shot multiplicity.  `handle expr with h` validates an implicit source-only thunk
fact and preserves the exact handler-use token span; it does not create a closure, frame,
provider, Core term, or runtime dispatch route.  Scoped source validation carries lexical block
lets, anonymous-function parameters, match/if-let and direct `with_error` pattern binders, and
typed `do` lets/binds (a bind receives the monadic inner type).  Unsupported computations and
unsupported comprehension evidence continue to fail closed.

Focused current evidence: handler-row typing 12/12; checked-computation inference 19/19;
row-normalization 9/9; checked-handler declaration 9/9; handler Core-boundary 15/15;
handler-application facts 10/10; handler-use anchor/block scope 2/2; lexical source scopes 4/4;
extended lexical scopes 6/6; parser handler surface 14/14.  The source typechecker also passes
`cargo fmt --check` and strict `cargo clippy -p ash-typeck --all-targets --all-features -- -D warnings`.

Checked-computation inference now traverses `if` conditions and branches, `match` scrutinees and
arm bodies, and `if let` matched/then/else expressions. Each child row is normalized and
deterministically unioned while retaining its operation provenance;
`match` arms and `if let` then branches reuse ordinary pattern checking to construct a cloned,
pattern-local `TypeEnv` before inferring a child, so declared operations can consume pattern-bound
payloads without leaking bindings to sibling branches. Unsupported inferred children, scrutinees,
and patterns retain their own source anchors. This remains fail-closed, immutable typechecker
evidence: it creates no source thunk, Core term, admission artifact, provider/handler frame,
dispatch, or runtime behavior. General handler lowering, continuation/resume and `done` semantics,
residual-row realization, and TASK-2014 production admission remain deferred.

The handler-application prewalk now also accepts canonical ambient `do` plain and record binds
before ordinary function-body checking. Those binds are plain values rather than monadic target
values; the regression proves that the prewalk no longer rejects them merely because it runs ahead
of ordinary body checking. It adds source typechecking evidence only, not Core/CPS lowering,
admission, frames, provider dispatch, or runtime behavior. See
[`task_1865_1878_ambient_do_bind_prewalk.rs`](../../../crates/ash-engine/tests/task_1865_1878_ambient_do_bind_prewalk.rs).

The local legacy pattern-checking entry point likewise now preserves positional payload types for
a parsed tuple ADT pattern such as `RuntimeError(code, message)`. This fixes source typechecking
for that legacy expected-type path; richer tuple/match source forms still require validated typed
Core lowering and consequently remain closed at the TASK-2014 admission boundary. See
[`task_1890_tuple_pattern_legacy_typecheck.rs`](../../../crates/ash-typeck/tests/task_1890_tuple_pattern_legacy_typecheck.rs).

The general source-to-Core lowering remains deliberately deferred. The existing Core bridge is an
inspection-only, narrow carrier and must reject checked facts it cannot represent (including the
general multi-clause/done/residual-row cases); it is not a production lowering path. Under the
selected TASK-2014 Path B, a form without validated typed lowering must reject at admission rather
than fall back to direct evaluation. No engine admission artifact, provider/handler frame
installation, async interpreter dispatch, or runtime handler execution is authorized by the
current source-typing evidence.

### Completed handler-only expected-type implicit-thunk specialization

For `handle expr with handler` only, input validation now retains the unification substitution
between the handler's expected computation result and the implicitly thunked operand result while
the operand's fresh inference variables are still live. It applies that substitution immediately
and stores the specialized result type under the handled `Expr` span. The later immutable-fact
publication re-infers the operand, so it retrieves that span-keyed specialized type instead of
carrying raw fresh-variable identities across passes.

The focused `collect_sleep` control specializes an implicitly thunked empty list to `List<Int>`
from the handler's expected input while preserving its exact declared `TestClock::sleep` row. This
does not add general call inference or thunking: ordinary calls never enter this handler-only
validation/publication path. It creates no closure, Core term, provider/handler frame, admission
artifact, dispatch, or runtime behavior.

Evidence is
[`task_2013_handler_application_fact.rs`](../../../crates/ash-typeck/tests/task_2013_handler_application_fact.rs)
(`task_2013_handler_expected_input_specializes_an_implicit_thunk_without_general_call_inference`).

### Derived-handler source application

The local `derive handler name;` fact is now usable by `handle expr with name` without inventing a
`TypeEnv` variable signature: normal value-namespace handler-marker resolution is followed by
checked-fact validation. The derived identity fold instantiates its fresh answer `A` from the
operand result and its open residual `r` from the operand's actual normalized row. It peels each
derived impl operation once, retaining concrete residual requirements and a real open tail with
provenance. The resulting application preserves canonical normalized row order and both
operand/handler anchors; a marker without a checked fact rejects, and lexical shadowing cannot
reuse an outer row fact.

The focused evidence is
[`task_2013_handler_application_fact.rs`](../../../crates/ash-typeck/tests/task_2013_handler_application_fact.rs).
It covers all currently parseable row forms, including aliases/groups and open tails, only through
the narrow explicit zero-argument call of a row-annotated parameter. In particular, the grouped
open-row control normalizes `OpenClockGroup` to the concrete `TestClock::Clock::sleep` identity,
retains the `rest` tail and its provenance in both the input and residual facts, and grants no
authority. The separate Core-inspection control typechecks an equivalent grouped open residual but
rejects it deterministically as a nonempty/open output row before selecting a source clause or
constructing Core. Unsupported computations remain fail closed. These controls add neither
Core/CPS lowering nor a provider/handler frame, engine/CLI or runtime behavior, or admission
authority; they also do not settle general continuation invocation or multiplicity policy beyond
the documented direct-resume cases. TASK-2014 Path B remains separately unimplemented.

## Current Cross-Cutting Source Controls

TASK-1005 now recognizes `true` and `false` together as the complete finite top-level `Bool`
universe for source match exhaustiveness, and a one-arm `true` match reports the missing `false`
witness. Other primitive scrutinees remain conservative: a literal-only `Int` match still requires
a wildcard/default diagnostic. This is source-typechecker evidence only; it does not lower a
general Boolean or match expression to Core/CPS. See
[`task_1005_match_exhaustiveness.rs`](../../../crates/ash-typeck/tests/task_1005_match_exhaustiveness.rs).

TASK-786's public-builtin `await` parser control retains the ordinary `process_handle` parameter
name while continuing to reject reserved `handle` in that position. Its legacy `Proc<A>` spelling
is metadata/parser evidence, not a restored computation carrier: the public-signature control
continues to reject `Proc` as unresolved, consistent with TASK-2000's removal of public
`Act`/`Proc` registration and bridges. This changes neither handler syntax nor handler/runtime
semantics.

## TASK-2014 Path B Handoff

TASK-2014 now selects checked Core/CPS as the sole production owner for admitted source programs.
When this task realizes a source form, it must contribute concrete operation identity, checked
handler clauses, normalized residual rows, and source anchors to the explicit admission artifact.
It must not infer provider or frame authority from those rows. Provider bindings and authorized
frame-install instructions are separate artifact fields, and their execution must preserve
TASK-1993 innermost-first handler/provider lookup.

The implemented production exceptions are strictly local `absorb_sleep` and exact abortive
`trap_sleep` handlers over `TestClock::sleep(Int) -> Int`. `absorb_sleep` has direct `resume(ms)`,
identity `done`, and literal `0`; `trap_sleep` has the same exact operation application but its
operation clause does not invoke `resume` and instead lowers fixed `1 / 0`, producing a
post-admission language trap. Each executes only after a prior same-Engine check, canonical parsed
anchor/Core comparison, typed Core/CPS validation, and one root `SourceHandler` instruction.
That instruction authorizes one engine-private checked-CPS handler installation/dispatch; rows do
not install it. These tokens admit no provider binding, provider frame, deep/generalized handler
semantics, or multi-frame chain. `Engine::execute`, generic V1 evidence, CLI trace/runnable
helpers, and all other handler forms remain closed; only the exact `trap_sleep` `ash run --format
json` route projects its V1 trap.

The remaining lowering work therefore includes general handler bodies, continuation/resume and
`done` semantics, and admissible residual-row realization. It must also support canonical terminal
envelope outcomes at the future production boundary: return, missing admission,
malformed/unchecked Core, handler-body trap, timeout, and cancellation. Until then, every source
form beyond this sealed fixture is closed at admission with no legacy direct-evaluator fallback.

## Approved Deep-Affine Continuation Semantics

TASK-2013 adopts the target rule in SPEC-099b §5. Checked handler clauses match in source order;
the first matching concrete operation clause receives an affine `resume` that may be used zero or
one time. A zero-use clause is abortive. A one-use `resume(value)` runs the captured continuation
with the same handler reinstalled at its original stack position, so a later operation in the
resumed tail is again handled by that handler. The surrounding stack retains TASK-1993
innermost-first handler/provider lookup.

Normal completion of the handled computation, including a normally completed resumed tail, goes
through `done` exactly once. Residual rows remain structural checked facts: they remove only the
handled concrete identities and retain remaining ordered/open-tail structure; they neither select
nor install handler/provider frames. This decision supersedes the historical shallow-frame wording
in SPEC-099b and does not authorize a generic frame or provider route.

The Engine witness is now admitted and executed through checked Core/CPS: checked ordered
`sleep → wake → resumed sleep` facts, a closed structural residual row, one source anchor, and
explicit authorized `SourceHandler` frame instructions produce `Int(107)`. Its two `resume` uses
are one per clause; the resumed tail re-enters the deep handler, and normal completion applies
`done(value) => value + 100` once. It preserves the shallow behavior of the existing
fixtures outside this explicit deep route. It does not complete arbitrary clause patterns,
multi-shot continuations, imported/generic handlers, arbitrary frame chains, or broad CLI parity.
