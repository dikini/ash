# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Common Changelog](https://common-changelog.org/).

## [Unreleased]

### Added

- TASK-712: added `proc::par` and `proc::scatter` all-or-none child admission across `std::proc`, type checking, and interpreter runtime, including ordered child registration/handle return, deferred child-failure observation via later `proc::await`, rollback on admission failure, and tuple-style numeric handle projection compatibility for `proc::par` results.

- TASK-711: added `proc::yield() -> Proc<Unit>` across `std::proc`, type checking, and interpreter forcing, including cooperative scheduler-yield runtime support, process-identity preservation coverage, and regression/proptest checks that yield introduces no child-process or handle-observation side effects.

- TASK-710: added affine runtime `P<A>` process handles and `proc::await`, including single-consumption observation, retained terminal-state projection, structured child-failure surfacing with preserved lower causes, and workflow-path runtime-state propagation for Proc await forcing.

- TASK-709: introduced the interpreter process registry and component-wise child environment projection substrate, preserving `ProcessId` parent/child identity, write-once terminal process state, and equal-or-narrower child role authority by capability name/effect/constraints without replacing workflow `ControlLink` supervision.

- TASK-708: implemented expression-level operational `fail` and scoped `with_error` handling across parser/lowering, type checking, and interpreter runtime, keeping operational failures distinct from ordinary Ash `Result::Err` values and preserving lower failure cause context when handlers re-fail.

- TASK-718: added the initial `std::proc` library surface and runtime stubs for `proc::unit`, `proc::bind`, and `proc::then` over opaque `Proc<A>` values without creating child processes, `P<A>` handles, scheduler behavior, or `from_act` embedding.

- NOTE-009: exploratory design note for capability interfaces, Ash-defined capability implementations, resource types/instances/bindings, internal authority, authority provenance, and late binding between interfaces, implementations, and concrete resources.

- TASK-707: registered opaque builtin `Proc<T>` and `P<T>` type constructors in `ash-typeck`, preserving generic process constructor annotations through type conversion and rejecting malformed process constructor arities without adding runtime process operations.

- TASK-706: added `ash-core` runtime identity and failure carrier substrate for Phase 98, including `RunId`, `ProcessId`, crate-internal `BranchId`, `LexicalFrameId`, `EffectScopeId`, process lifecycle/terminal carriers, structured operational/process failure carriers, and skeleton workflow failure/report carriers without wiring runtime admission or Proc operations.

- SPEC-047: Act Monad specification (draft). Defines `Act<A>` type constructor, `act {}` block expression, `invoke`/`unit`/`bind` builtins, effectful function declarations, purity enforcement, and the unification of pure expression evaluation with effectful workflow execution. 33 tasks across 4 tracks (TASK-672 through TASK-704). Related plan: PLAN-097.

- TASK-683: introduced the runtime-only `ActEnv` carrier in `ash-interp` with explicit construction from runtime state/capability context, policy evaluator, provenance, and effect log state; kept it out of `ash_core::Value` and added regression coverage for the runtime boundary.

- TASK-684: routed expression-level `invoke(...)` through a dedicated runtime primitive path under `Expr::Call`, returning a closure-shaped Act value that captures provider/action/args while preserving existing pure builtin dispatch. (TASK-684)

- TASK-685: added closure-backed execution support for lowered `Act<T>` shapes via runtime `unit`/`bind` sequencing, plus regression tests proving lowered act blocks execute through the interpreter. (TASK-685)

- TASK-686: bridged workflow execution into the Act runtime boundary by constructing `ActEnv` from workflow runtime state, policy evaluation, and provenance on entry; added coverage to verify the workflow bridge reuses the existing capability context without regressing workflow-level act semantics. (TASK-686)

- TASK-677 through TASK-680: Act monad type system integration. `Act` registered as unary type constructor `* -> *`. `Expr::ActBlock` type-checked with monadic bind/pure-bind/return semantics. `invoke(provider, action, args)` recognized as `Act<Value>`. Purity enforcement rejects `act {}` blocks and `invoke(...)` calls in pure `fn` bodies; both allowed when return type is `Act<T>`. (TASK-677, TASK-678, TASK-679, TASK-680)

### Fixed

- TASK-708: tightened `fail` / `with_error` keyword-boundary parsing so those contextual forms no longer consume legal identifier prefixes such as `fail_count` or `with_error_handler`.

- TASK-708: `fail` now attributes operational failures to the current runtime tower/identity (`LexicalFrameId`, `EffectScopeId`, or `ProcessId`) instead of hard-coding pure lexical failures, and exact identifier spellings `fail` / `with_error` are now reserved consistently across declarations and expressions.

- CLI module-file fallback now ignores `workflow` mentions in line comments, so `ash check std/src/lib.ash` reports the stdlib root as a module file instead of surfacing a generic workflow parse error.

- Typeck/lowering contract alignment for act-block structural validation: `check_expr` now enforces the same empty/requires-return/return-must-be-last contract as `lower_act_block`, closing an end-to-end semantic mismatch where typeck would accept shapes that lowering rejects.

- Purity enforcement for nested `Expr::FnDef` bodies now computes `allow_effects` from the nested function's own return type annotation rather than inheriting the enclosing function's flag, so `fn(x) -> Act { act { ret x; } }` is legal inside a pure outer function body.

- TASK-681: 56 tests proving Phase 97's `Act<T>` typing is additive — Type::Fun construction, non-unification with Type::Fn, non-collapse with Type::Constructor, substitution independence, and proptests. (TASK-681)

- TASK-682: 13 tests for Act<T> inference (String, Bool, chained binds), purity rejection via check_expr and check_purity, and proptests for type inference invariants. (TASK-682)

- PLAN-097: Phase 97 Act Monad implementation plan is now closed out and reconciled with the landed task breakdown. Track A (surface/core), Track B (type system), Track C (runtime), and Track D (specs/library-validation) total 71 hours in the final plan framing.

- NOTE-006: workflow ambient typing and runtime failure boundary. Records the current design direction that workflows still produce `Act<A>`, workflow typing tracks structured ambient-context projections (`capabilities`, `plays role`, `requires`, `ensures`) rather than raw `ActEnv`, and runtime execution reports `Result<A, WorkflowFailure>` without prematurely committing to supervisors or orchestration-specific recovery semantics.

- DESIGN-030 and SPEC-048: proc library and minimal runtime substrate draft packet. Define `Proc<A>` as a distinct process-structured computation type with a library-first `proc` surface (`unit`, `bind`, `then`, `par`, `scatter`, `gather`), keep workflow compatibility explicit, and defer runtime-heavy features such as `run`, mailbox/channel mechanics, and spawning.

- NOTE-007 and NOTE-008: runtime environment and operational bottom/failure design notes for the Act/Proc/Workflow tower. Capture identity-indexed typed component lookup, EffEnv vs ProcEnv boundaries, initial access modes, effect-failure channel, `fail` as operational bottom, multi-arm `with_error`, and async `par` failure observation via process handles.

- SPEC-049, SPEC-050, and SPEC-051: normative draft specs for process runtime semantics, operational bottom/scoped handling, and initial workflow semantics. The new specs promote the resolved `Proc<A>`/`P<A>` process model into process identity, affine/linear handle, child environment projection, `yield`, `await`, wait-for-all `join`/`gather`, tower/entity-indexed `fail`/`with_error`, process-observation failure aggregation, workflow admission/governance, `WorkflowFailure`, reporting, and lower-failure reinterpretation contracts.

- PLAN-098: Proc, process runtime, failure, and workflow boundary implementation plan. Adds substrate-first tasks TASK-705 through TASK-718 for runtime identities, operational `fail`/`with_error`, `Proc`/`P` type registration, `Proc` core combinators, process handles, `yield`, `par`/`scatter`, `await`/`join`/`gather`, workflow boundary reports, and cross-layer validation.

### Changed

- Completed TASK-705 semantic tower runtime preflight for Phase 98 after merging current `main`; baseline fmt/test/clippy gates are green, TASK-706 may proceed as carrier-only work, and Act-dependent Proc slices remain deferred until their specific Act prerequisites are needed (TASK-705).

- DESIGN-030 and SPEC-048 now record the current semantic-environment lattice `Pure < Effectful < Proc < Workflow`, clarifying that capability/provider and policy admissibility begin in the Effectful/Act stratum, proc adds split/join/process-local runtime semantics, workflow adds governance metadata and failure/reporting semantics, operational availability flows top-down from outside/workflows to processes to effects to pure functions, environment component lookup is identity-indexed by workflow/process/branch/effect/lexical frame identity, and async `par` returns running process handles `P<A>` rather than a synchronous result pair or special join object.

- DESIGN-030 previously recorded the resolved `par` semantics slice; SPEC-048, SPEC-049, and SPEC-050 now split that slice across public surface, process-runtime, and operational-failure ownership: `par` creates child `ProcessId`s, derives child environments by typed projection instead of context cloning, limits `par`-site handlers to start/admission/handle-creation failures, treats `P<A>` as a first-pass affine/linear process handle, defines `await` as the single-handle observation primitive, defines `join`/`gather` as wait-for-all observation barriers with aggregate failure preservation, and adds `yield : Proc<Unit>` as an explicit cooperative scheduling point.

- DESIGN-030 removes the stale synchronous-`par` open question, includes `join` in the initial proc library surface, records NOTE-007/NOTE-008 as the current environment/failure design-note layer, and states that workflow needs a separate semantics spec rather than only surface-syntax tracking.

- SPEC-004 now cross-references SPEC-050 as the normative operational-bottom authority, resolving the prior note that surfaced `Pure` bottom was future work while preserving SPEC-004's existing workflow effect-classification lattice.

- TASK-689 through TASK-691 are now complete in the Phase 97 worktree: `std/src/act.ash` no longer relies on placeholder public helper builtins, ordinary-library `guard` now forces policy decisions through the internal `act::__guard` bridge at Act-force time, focused engine/interpreter validation covers import/type/execute plus async-force boundary behavior, `.gitignore` now ignores the standalone `crates/ash-bench/target/` output, and `ash-bench` carries an approximate `phase97_act` Criterion smoke baseline for desugared Act execution (`guard_force_permit` ≈ 5.6 µs; bind-chain force depths 1/4/8/16 ≈ 9.8/51.7/107/226 µs).

- TASK-689D is complete for the public opaque `Act` boundary. The now-superseded exploratory/probing slices established the preferred A-path (`builtin type ActEnv`; ordinary `type Act<A> = ActEnv -> (ActEnv, A)`), hidden-carrier enforcement, hidden runtime `ActEnv` threading, `invoke(...)` dispatch through that hidden carrier, async Act-force support across the relevant workflow/expression surfaces, Send/Sync storage cleanup, and stream-backed workflow entry coverage. `std::act` now exposes ordinary `unit`/`bind`/`then`/`guard` helpers over hidden bridge builtins; the remaining token/list force-result shape is documented as an internal compatibility detail for follow-on native effect-runtime work rather than as a public representation or a TASK-689D blocker.

- TASK-689E is now complete: the engine/type boundary distinguishes public type identity from public constructor visibility. Plain `type` definitions now remain importable/discoverable for signatures and type annotations without auto-exporting constructors, while `pub type` continues to expose constructors/representation. TASK-689D is now unblocked as the next opaque-`Act` follow-on.

- TASK-689B now preserves imported ordinary `pub fn` signatures for `std::act` through module loading and engine type binding. `Workflow` carries imported ordinary-function signatures, `build_imported_closures(...)` threads them across the engine boundary, `bind_imported_callable_types(...)` binds them with `ash_typeck::fn_signature_type(...)`, and focused ash-engine coverage now verifies the upgraded internal binding path.

- TASK-689A now documents and tests the real `std::act` boundary honestly: `check_module_file` still accepts `std/src/act.ash`, and ordinary import-backed engine execution can now resolve `use act::{unit, bind, then, guard}` through the real engine path. TASK-689 has since closed that loop by removing the placeholder public helper builtins and aligning the public surface with the ordinary-library contract promised by SPEC-047.

- TASK-689C is now complete: `ash-typeck` supports record field projection, projected callable invocation now parses/typechecks/evaluates honestly, and Phase 97 gained a narrow `act::policy_check` bridge that preserves the runtime-only `ActEnv` boundary while allowing `std::act::guard` to be implemented as an ordinary library function.

- Phase 97 design laws are now made explicit in SPEC-047: `Act` is the outer marker of effectfulness, `Act<Result<A, E>>` is the preferred conventional shape for effectful computations with domain failure, `Act` remains representationally opaque and eliminable only through effectful contexts, and workflows are intended to converge toward richer constructs built on top of effectful functions rather than a separate sequencing foundation.

- Phase 97 Track D is now fully closed out: TASK-689A established an honest substrate for ordinary library helpers, TASK-689B preserved imported ordinary `pub fn` signatures for `std::act`, TASK-689C landed the policy/environment substrate for an honest ordinary-library `guard`, TASK-689E refined opaque public type identity exports, TASK-689D completed the public opaque `Act` boundary and hidden-carrier runtime proof, TASK-689 removed the remaining placeholder helper surface, TASK-690 validated parse/type/execute behavior end to end, and TASK-691 recorded the approximate benchmark smoke baseline.

- TASK-688: finalized the Phase 97 SPEC-047 amendment set with targeted downstream spec updates for surface syntax, type-system coexistence, operational semantics, purity boundaries, and first-class-function dispatch notes. (TASK-688)

- Phase 97 TASK-672 is now complete. SPEC-047, PLAN-097, and the Phase 97 PLAN-INDEX packet are aligned around the additive architecture: surface-only `act { ... }`, lowering into existing core expressions, `invoke` as a runtime primitive callable via `Expr::Call`, `unit`/`bind`/`then`/`guard` as library functions, and no Phase-97 SPEC-025 expansion.

- Baseline verification gates are green again for Phase 97 worktree execution. Repaired pre-existing workspace blockers by restoring `process::run` builtin dispatch compatibility for existing interpreter tests, aligning provider/test files with `cargo fmt` and strict clippy, and hardening a parser debug test fixture path/expectation so `cargo test --all`, `cargo fmt --check`, and `cargo clippy --all-targets --all-features -- -D warnings` pass cleanly.

- TASK-673 act-block lowering now respects declared effectful names when deciding whether to wrap bind RHS in `unit()`, so user-defined effectful calls are preserved as monadic values instead of being misclassified as pure.

- TASK-673 surface Act substrate is now landed: `surface::ActStmt` and `surface::Expr::ActBlock` are present as span-carrying parser/lowering carriers without introducing a new core IR act-block form.

- Engine callable lowering now propagates module/program effectful-name context through local and imported user-defined function bodies, closing the remaining Phase 97 act-block gap where effectful RHS calls could still be mislowered outside workflow-body lowering.

- Phase 97 Track A surface/lowering slice is now landed for TASK-674 through TASK-676. `parse_expr::expr()` accepts only braced expression-level `act { ... }` blocks with bind/return statements, lowering desugars `Expr::ActBlock` into existing `unit(...)`/`bind(...)` + closure core forms, and `ash-parser` now carries focused regression/property coverage for nesting, invalid sequences, and workflow-vs-expression `act` disambiguation.

- NOTE-005 status updated: design exploration now has a normative spec counterpart (SPEC-047).

- Phase 96 Track A: Module resolution and stdlib integration (TASK-655 through TASK-659). Module resolver now supports cycle detection via visiting set. Stdlib modules (string, list, predicate, result, option) resolve through builtin stdlib root. CLI run command routes ordinary files through `engine.run_file()` for full import resolution. Entry bootstrap path preserved and verified. 12 module resolution + 13 entry bootstrap tests pass.

- Phase 96 Track C: Capability providers (TASK-666 through TASK-668). HttpProvider with get/post/put/delete/head, configurable timeout and host allowlist. TimeProvider with now/now_iso/epoch_millis/sleep and mock time support. ProcessProvider converted from `builtin fn` to capability per three-pillar principle -- timeout, command allowlist, stdout+stderr+exit_code capture. 22 + 21 + 21 = 64 provider tests.

- Phase 96 Track D: Testing and auditing (TASK-669 through TASK-671). 8 multi-file e2e tests (cross-file pub fn, type imports, nested modules, stdlib shadowing, gap documentation). 21 capability boundary audit tests (effect levels, unknown action rejection, argument validation, security allowlists, observe/execute boundary). 6 performance baseline tests (engine build <5ms, simple workflow <5ms, stdlib import <50ms).

- Phase 94: Ash wiki pilot classification slice (TASK-647). Created
  `docs/wiki/indexes/pilot-authority-map.md` and
  `docs/wiki/indexes/pilot-supersession-map.md` classifying the LSP/tooling
  cluster (SPEC-038 through SPEC-043, Phases 84-89) against the SPEC-045
  authority/status/health model. Identified 6 friction points.

- Resolved FP-1: Renumbered SPEC-021-LEAN-REFERENCE to SPEC-046, eliminating
  the SPEC-021 numbering collision with SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.
  Updated 22 files, 45 references. No runtime-observable references changed.

- Resolved FP-6: Renumbered PLAN-035-generic-builtin-fn to PLAN-037, eliminating
  the PLAN-035 numbering collision with PLAN-035-INCREMENTAL-ANALYSIS.
  Updated 4 files.

- Resolved FP-3: Documented the Ash spec `draft` convention in SPEC-045 §7.2
  rule 5. Specs use `status: draft` even after implementation; the wiki
  metadata model treats these as accepted and governing unless superseded.

- Phase 90 Track A: `spec_processor` crate — repository analysis pipeline
  for Ash plan/spec documents. Implements file collection (`collect.rs`),
  shared finding types with `Tier` enum (`finding.rs`), plan-index coherence
  checker (`plan_index.rs`), changelog completeness checker (`changelog.rs`),
  spec cross-reference validator (`spec_links.rs`), and report aggregator
  with human-readable and JSON output (`report.rs`). 49 tests across 6
  suites. All functions use `Result`-based error handling, `LazyLock` regex
  caching, and comprehensive documentation (TASK-590 through TASK-599).

- Phase 90 Track B: `std::json` builtin module with `parse`, `stringify`, and
  `stringify_pretty` functions backed by `serde_json` (TASK-597).
  Validates and transforms JSON strings via the evaluator builtin dispatch path.
- Phase 90 Track B: `std::process` builtin module with `run` function for
  subprocess execution via `std::process::Command` (TASK-598).
  Returns stdout as a string. 8 integration tests.
- Phase 90 Track B: `std::markdown` builtin module with `parse` function backed
  by `pulldown-cmark` (TASK-596). Parses CommonMark into a JSON AST string with
  `heading`, `paragraph`, and `code_block` block types. 8 tests.

- Phase 90 Track C: `spec_processor` integration and CI gate. Added four modules:
  `example_check.rs` (parse+type-check `.ash` files via `ash-engine` API, emitting
  `ExampleFailure` on errors — TASK-600), `capability_boundary.rs` (declare and
  audit 7 expected stdlib capabilities, emitting `ToolingGap` for missing stubs
  — TASK-601), `meta_validation.rs` (self-audit processor source tree, doc
  cross-references, capability consistency, and test coverage — TASK-602), and
  `pipeline.rs` (orchestrate all 7 check modules into a single `run_pipeline()`
  entry point returning a `Report` suitable for CI gating — TASK-603). 63 tests
  across 10 suites (2 ignored for real-repo manual verification). All review
  findings addressed: `Result`-based error propagation (no panics), private
  `PipelineError` fields, explicit `match` on all file reads, `and_then` for
  flattened error chaining, `starts_with` for declaration detection.

### Fixed

- Removed unnecessary hash in raw string literal in `expr_let_integration.rs`
  (clippy `needless_raw_string_hashes`).

- Phase 95: `Expr::Let` — pure expression let-binding in core IR. Added
  `Expr::Let { pattern, expr, body, span }` to `ash_core::ast::Expr` for pure
  scope extension in fn bodies (TASK-648). Lowerer desugars `Expr::Block` to
  nested `Expr::Let` (TASK-649), deleting the `normalize_imported_callable_expr`
  workaround from `module_loader.rs`. Evaluator implements EXPR-LET via child
  context scope extension (TASK-650). ANF lifter and monomorphizer handle
  `Expr::Let` (TASK-651). 7 integration tests covering inline fn, top-level fn,
  nested let, closure capture, list patterns, and variable shadowing (TASK-652).
  Fixed `and`/`or` short-circuit evaluation per SPEC-004 EXPR-AND-FALSE and
  EXPR-OR-TRUE (TASK-653).

- Phase 95 code review fixes: replaced dead `BinaryOp::And`/`Or` arms in
  `eval_binary_op` with `unreachable!()` guard (short-circuit handled in
  `eval_expr`). Added `LetPatternBindFailed` error variant for Expr::Let
  pattern-match failure (SPEC-004 `PatternBindFailure`), replacing misused
  `NonExhaustiveMatch`. Added 2 integration tests: runtime pattern-match
  failure in fn let-binding, and pub fn with let-sequencing via `parse_file`
  (9 total e2e tests for Expr::Let).

- Phase 95 spec review fixes (TASK-648/649/650): added `span: Span` to
  `Expr::Let` in SPEC-001 §2.6, TASK-648, and TASK-649 desugaring sketch for
  pattern-match-failure diagnostics. Fixed TASK-650 eval sketch to use child
  context (`ctx.extend()`) matching existing `eval_match`/`eval_if_let` pattern
  instead of parent-scoped mutation. Clarified TASK-649 module_loader deletion
  flow: raw surface `Expr::Block` stored in `InlineCallable::body` is desugared
  at lowering time, unifying all three code paths.

- Ash wiki architecture docs and rollout scaffolding: added FUTURE-004, DESIGN-029, SPEC-045, the initial implementation plan, a concrete metadata schema reference, a shared corpus-analysis substrate design note, and Phase 94 task/PLAN-INDEX scaffolding for the static-first human/AI shared knowledge substrate over the Ash corpus. The new documents define authority/status/health semantics, metadata carrier rules, supersession and drift/audit models, onboarding/library-service goals, staged rollout for static views/query workflows/service exports, and practical reuse boundaries with the spec processor and `ash-lint`.

- Phase 93 generic builtin fn (TASK-634 through TASK-644): imported `builtin fn`
  declarations now carry full type signatures through the module loader and
  engine typecheck pipeline. `InlineCallable` preserves `BuiltinFnDef` signatures;
  `Engine::check()` uses `builtin_fn_signature_type()` for precise polymorphic
  types instead of arity-only synthetic types. `std/src/list.ash` declares
  `len`, `head`, `tail`, `append`, `concat`, `filter`, `map` with generic
  type parameters. `std/src/predicate.ash` declares `is_int`, `is_string`,
  `is_bool`, `is_list`, `is_record`, `is_null`. Qualified dispatch entries
  (`list::len`, `predicate::is_int`, etc.) added to `builtin_dispatch_table()`.
  End-to-end verification: import, typecheck, execute all pass.

- TASK-636: audit confirmed type-variable freshening is unnecessary.
  `instantiate_fn_call` creates fresh `Substitution` per call; sequential
  polymorphic calls with different concrete types typecheck independently.

- TASK-629: removed the legacy regex capability carrier and engine wiring now
  that imported `std::regex` calls are proven through builtin declarations and
  evaluator dispatch. Provider-era regex tests were dropped in favor of the
  existing builtin-path coverage in `ash-engine` and `ash-interp`.

- Track E closeout proof (TASK-630): positive end-to-end `std::regex` coverage
  now explicitly proves module import, typechecking, evaluator dispatch, and
  runtime execution for imported builtin regex calls. The historical
  `regex_import_limitation` test target remains only as a stable command name
  and now covers honest positive/complementary regression behavior.

- Track E implementation (TASK-627, TASK-628): stdlib `regex` builtin imports
  now execute through evaluator dispatch for `regex::find`, `regex::matches`,
  and `regex::replace`. `ash-interp` now owns the runtime regex behavior using
  the `regex` crate directly, preserving clear invalid-pattern errors.

- Track D1 implementation (TASK-623, TASK-626): `std/src/string.ash` and
  `std/src/record.ash` stdlib modules with `builtin fn` declarations, making
  `concat`, `starts_with`, `ends_with`, `is_empty` (string) and `keys`,
  `values`, `record` (record) importable via the module system. Extends
  `CallableKind::Builtin` to carry a `module` name so qualified dispatch routes
  correctly through the evaluator. Context closures now take priority over
  unqualified builtins in `eval`, and `builtin fn` names no longer misparse as
  capability action targets in the parser.

- Track C implementation (TASK-621, TASK-622): runtime builtin dispatch table
  and clear error on unknown builtins. Adds `BuiltinEntry` metadata struct and
  `builtin_dispatch_table()` in `ash-interp` mapping qualified names to arity,
  variadic flag, and implementation status. When `eval_function_call` returns
  `UnknownFunction` for a name in the dispatch table, produces
  `EvalError::UnimplementedBuiltin` instead. 23 new integration tests.

- Track B implementation (TASK-618 to TASK-620): `builtin fn` module loader
  and typechecker support. Introduces `CallableKind` enum (`User { body }` vs
  `Builtin`) to distinguish bodyless builtins from Ash-bodied functions. Module
  loader registers `builtin fn` exports, typechecker resolves their type
  signatures as `Type::Fn(params, ret)`. D2 decision gate passed: full
  import/typecheck pipeline works for bodyless functions. 11 new tests.

- Parser support for `builtin fn` declarations (TASK-615). The parser now
  recognizes `[pub] builtin fn <name>[<T>](<params>) -> <Ret>;` as a new
  definition form, producing `Definition::BuiltinFn(BuiltinFnDef)`. Return
  type is mandatory; braces are rejected with a parse error. Dispatch is
  added in both inline-module and file-level definition loops, with correct
  priority over plain `fn`. Includes 10 integration tests covering valid
  forms, error cases, and module-level dispatch.

- `builtin fn` declaration form: design note, spec, and implementation plan.
  Three new documents establish pure runtime-provided functions as a first-class
  declaration form, closing the gap between `pub fn` (Ash bodies) and capability
  providers (effectful operations). Includes three-tier classification (strictly
  monomorphic / ad-hoc polymorphic / parametric polymorphic), full
  backward-compatibility contract for all 21 current evaluator builtins, and
  7-track plan (A through F).

- Track A implementation (TASK-614 to TASK-617): `builtin fn` parser and
  surface AST. Adds `BuiltinFnDef` variant, semicolon-terminated parsing,
  lowering to core IR, and module loader snippet extraction. Decision gate D1
  passed. Review fixes: private builtin visibility (SPEC Section 5.3), hover
  text alignment, body-rejection error severity (Cut), spec-required error
  tests (SPEC Section 11). Phase 92 added to PLAN-INDEX.

- Non-blocking doc clarifications: `extern fn` wording tightened with explicit
  scope (link-time resolution, ABI constraints, effect rules), InlineCallable
  consumer sites named concretely (evaluator, import resolution, typeck
  registration), regex carrier-vs-semantics note added distinguishing
  current `Operational` provider artifact from intended pure classification.

### Removed

- TASK-643: deleted `add_builtin_functions()` from `ash-typeck/src/type_env.rs`.
  List builtin type signatures are now provided exclusively through `.ash`
  declarations via `Engine::check()` -> `builtin_fn_signature_type()`.

### Fixed

- Reverted `role` to `sender` field name in LLM stdlib Message type. `role` is
  a reserved keyword in Ash, causing parse failures in pattern matching and
  struct literals. All occurrences in `types.ash`, `prompt.ash`, `mod.ash`,
  `lib.ash`, and the Rust provider (`chat.rs`, `tool_dispatch.rs`) now use
  `sender`. The inspector function was also reverted from `role()` to
  `sender()` and the helper from `role_name()` to `sender_name()`.

- PLAN-INDEX: Phase 57 status corrected from stale "Ready" to "Done" -- all
  57A (SPEC) and 57B (implementation) tasks were already complete including
  closeout TASK-369. Only TASK-368b (extended entry-point tests requiring
  io::Stdout capability) remains deferred to a future phase.

- Removed dead `timeout_ms` and `max_retries` fields from `LlmConfig`. These
  were declared but never wired to the async-openai client, making them
  misleading configuration surface. Also replaced bare `.lock().unwrap()`
  with `.lock().expect("descriptive message")` in `stream_storage.rs` for
  all Mutex acquisitions to provide actionable panic diagnostics.

- PLAN-INDEX: Phase 48 status updated from "Partial" to "Done" -- all remediation
  tasks (TASK-318, TASK-311, TASK-319) completed in Phase 49. Phase 92 status
  updated from "Blocked" to "Done" -- TASK-631B resolved by Phase 93 TASK-643.
  Phase 74 status updated from "Planned" to "Done" -- all 8 tasks complete.
  Phase 76B task statuses corrected from stale "Complete" to "Planned" with
  blocker documentation -- synthesized tests and small-world exploration require
  introspection and enumeration substrates that do not yet exist.
  Phases 84-89 status corrected from "Planned" to "Done" -- all tasks
  (TASK-570 through TASK-576, TASK-569) were already complete.
  Phase 77 (LLM Standard Library) status corrected from "Planned" to "Done"
  -- all 23 tasks (TASK-516 through TASK-538) were already complete.

- TASK-632: reconciled Phase 92 planning/changelog/task surfaces with the
  landed state. `PLAN-INDEX` now reports TASK-631A and TASK-632 as complete and
  keeps TASK-631B explicitly blocked on deferred D2 work; TASK-633 remained a
  separate full-workspace verification task rather than being overclaimed in the
  status-reconciliation pass.

- TASK-633: fresh full-workspace verification for the Phase 92 worktree passed:
  `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --check`, and `cargo doc --no-deps`. The doc build still emits
  pre-existing rustdoc warnings in `ash-engine` LLM-provider comments, but the
  command succeeds and the verification surface for Phase 92 is now current.

- TASK-631A: removed the hardcoded `ash-typeck` registrations for
  `string::concat`, `string::starts_with`, `string::ends_with`, and
  `string::is_empty` so imported stdlib string builtins resolve through the
  Track D1 declaration files instead. Deferred hardcoded entries such as list
  builtins and bare partial-application helpers remain in place; no record
  entries needed removal because they were already absent from the type env.

- Proptest flake in `capability_parser_props`: `valid_type_name()` strategy
  could generate `Fn`, a contextual keyword in type position (function-type
  syntax `Fn(T) -> R`). The `Fn` case now filtered out of the strategy.

- Phase 79 status drift in `docs/plan/PLAN-INDEX.md`. The phase header and
  TASK-545 through TASK-550 rows now consistently report `✅ Complete`,
  matching the already-complete progress tables and landed implementation.

- Clippy warnings in `ash-engine`: `parse_program_with_functions` missing
  `# Panics` doc section, `single_match_else` in workflow loop,
  `option_if_let_else` in regex provider, `too_many_lines` in
  `parse_workflow_source_with_imports` (extracted `process_program_definitions`
  helper and `ProgramProcessingResult` type alias), unnecessary raw string
  literal hashes in `regex_import_limitation` test.

- PLAN-INDEX phase status alignment. Phase 65 (8/8 tasks), Phase 67 (11/11
  tasks), and Phase 91 (7/7 tasks including TASK-612) now correctly show
  `✅ Complete` instead of stale `🟡 Ready`.

- Phase 90 status reconciliation (TASK-613). PLAN-INDEX.md previously marked
  TASK-590 through TASK-594 (Track A), TASK-596 through TASK-598 (Track B),
  and TASK-600 through TASK-603 (Track C) as ✅ Complete despite no code
  existing on `main`. All downgraded to 📝 Planned. TASK-595 (`std::regex`)
  downgraded from ✅ Complete to 🟡 Partial: the Rust provider is functional
  with 12 passing tests, but the Ash-language import surface is not proven
  end-to-end because `fn` bodies with `act execute` cannot yet be parsed at
  the expression level. TASK-599 (`std::diff`) remains correctly ⏸️ Deferred.
  TASK-613 closed as ✅ Complete. Stale worktree references removed from
  task file. TASK-595 file path and error-handling wording corrected.
  Limitation regression test added to codify the current import boundary at
  that time; this limitation was later removed by Phase 92 Track E.

- Phase 90/92 regex documentation alignment. Phase 90 surfaces now reflect
  that `std::regex` is proven end-to-end from imported Ash source via builtin
  declarations and evaluator dispatch. TASK-595 is restored to ✅ Complete,
  while legacy regex-carrier cleanup remained explicitly deferred to TASK-629.

- Phase 65↔91 alignment remediation (TASK-612). Bare qualified method
  syntax `Interface::method` without call parentheses is now rejected
  by the parser instead of silently accepted as a zero-argument call.
  Lowercase pseudo-variant patterns like `foo(bar)` and `foo { x: y }`
  are now rejected by the pattern parser instead of silently accepted
  as variant patterns.  `Propose.binding` is explicitly rejected in the
  MVP typechecker instead of accepted with a fabricated fresh type
  variable, restoring the TASK-423 documented contract.  Stale task/doc
  surfaces presenting record-shaped `RuntimeError { exit_code, message }`
  reconciled to the canonical tuple-variant form `RuntimeError(Int, String)`.

### Added

- Small-step interpreter with compressed IR (Stmt/Frame/Config) and full
  async execution engine (TASK-604).  Runs workflows via structural
  reduction instead of recursive evaluation.  18 small-step tests pass
  including workflow call with parameter binding.

- Statement lifting pass (ANF-style) for pipe operator support (TASK-605).
  Extracts effectful sub-expressions into synthetic `Let` bindings.  Pipe
  operator (`|>`) lexed, parsed, and lowered via partial-application
  desugaring.  2 end-to-end pipe operator tests pass.

- `Workflow::Call` runtime completion (TASK-606).  Big-step and small-step
  interpreters execute `Workflow::Call` with argument binding, arity
  checking, and unknown-target rejection.  `RegisteredCallableWorkflow`
  stores parameter names for runtime binding.  8 call-target tests pass
  across both interpreters.

- Small-step/big-step parity test corpus (TASK-607).  12 differential tests
  (`parity_*`) prove both interpreters agree on Done, Ret, Let, Seq, If,
  ForEach, Maybe, Must, and workflow call outcomes.  Zero divergences.

- Statement lifting contract hardening (TASK-608).  10 regression tests
  verify conservative preserve-original behavior for effectful expressions
  in unsupported positions (Ret, If condition, ForEach collection, guards,
  Send, Spawn, Split, Call arguments).  A sweep test covers all 29
  Workflow variants asserting no panics.  15 lift tests pass.

- Capability-registry effect classification (TASK-609).  Replaced hardcoded
  `EFFECTFUL_NAMES` list with `effectful_names_from_definitions()` that
  derives effectful names from declared `CapabilityDef`s in the program.
  `LoweringContext` carries the set; `lift_workflow_with_names()` threads it
  through the lifting pass.  Qualified calls and Spawn remain unconditionally
  effectful; unqualified calls are classified by declared capabilities.
  6 new classification tests; 21 lift tests total.

- Local helper workflow surface (TASK-611).  `Program` struct carries
  `helper_workflows`; parser supports multiple named workflows per file;
  engine registers helpers as callable targets with typechecker visibility.
  Helper parameter binding works at runtime in both interpreters.  5 engine
  integration tests pass including parameterized helper calls.

- `Workflow::Call` and `BinaryOp::Pipe` AST variants in `ash-core`.
  Compressed IR types (`Stmt`, `Frame`, `Config`) in `ash-core::small_step`.
  `lower_expr()` public API in `ash-parser`.

### Changed

- Lifting pass no longer panics on effectful expressions in unsupported
  workflow positions; preserves original expression for downstream
  diagnostics instead.

- Hardened helper-workflow follow-up fixes: synchronous callable workflow
  registration now works on current-thread Tokio runtimes without
  `block_in_place`; spawned child workflow failures are surfaced via explicit
  error reporting; lift variable numbering is reset per top-level lift pass;
  the type checker matches `BinaryOp::Pipe` defensively instead of falling
  through implicitly.

- Effect classification in lifting derived from capability declarations
  rather than hardcoded name list, eliminating false positives for
  user-defined functions that shadow stdlib names.

- Scoped-body lifting (Match arms, IfLet branches, FnDef bodies) now
  preserves the original expression when inner lifting produces synthetic
  bindings that cannot be hosted, instead of emitting unbound `__lift_`
  variable references (re-review B1 fix).

- Decide lowering returns `LoweringError::InvalidTarget` for legacy
  else-branch input instead of panicking (re-review B2 fix).

- Provider registry uses `std::sync::Mutex` instead of tokio async mutex,
  eliminating `blocking_lock()` panic hazard on current-thread runtimes.

- Pipe operator precedence tests: `a + b |> f` groups addition first;
  `x |> f(a, b)` prepends `x` as first argument.

- Lift regression tests corrected and expanded: Match arm test now
  asserts original expression preservation (not broken synthetic var);
  new IfLet and FnDef preservation tests added.

### MCP (Model Context Protocol) server bridge in new `ash-mcp` crate
  (TASK-569 Phase 4).  Built on `rmcp` v1.5, exposes 8 MCP tools that
  wrap `ash-lsp-core` analysis: `ash_get_diagnostics`, `ash_hover`,
  `ash_goto_definition`, `ash_complete`, `ash_document_symbols`,
  `ash_find_references` (placeholder), `ash_workspace_symbols`
  (placeholder), `ash_code_action` (placeholder).  Files are auto-opened
  on first tool call per SPEC-038 §8.5.  Responses include a one-line
  summary for token-efficient LLM consumption.  Stdio transport via
  `ash-mcp` binary.

- Go-to-definition and completion support in `ash-lsp-core` and `ash-lsp`
  (TASK-569 Phase 3).  `ash-lsp-core` gains a shared `position` module
  (byte-offset ↔ LSP Position conversion, token-at-offset extraction),
  `goto_definition` (identifier → definition span lookup across module
  declarations, nested definitions, and workflow entry), and `completions`
  (Ash keyword snippets + module definition name suggestions, excluding the
  token under the cursor).  `ash-lsp` wires `textDocument/definition` and
  `textDocument/completion` handlers with full `tower-lsp-server` ↔
  `lsp_types` boundary conversion.  14 new tests across both crates.

- Phase 87 Week 1 LSP foundation (TASK-569): new `ash-lsp-core` crate with
  a DashMap-backed VFS, incremental text change application, line/column ↔ offset
  conversion helpers, diagnostic aggregation (`ash-parser` + `ash-lint`), a
  version-aware analysis cache, keyword/top-level hover support, and symbol extraction.
  Added new `ash-lsp` binary crate with `tower-lsp-server` transport skeleton,
  stdio/TCP launch modes, working `didOpen` / `didChange` / `didClose` diagnostic
  publishing, hierarchical `textDocument/documentSymbol`, `textDocument/hover`, and
  service-level JSON-RPC tests covering diagnostics, hover, symbols, and close/change
  notification behavior.

- `ash-lint` library crate extracted from CLI binary (TASK-574).
  Public API: `lint_source`, `lint_module`, `lint_workflow`, `LintConfig`,
  `LintDiagnostic`, `LintCode`, `LintSeverity`, `LintFix`, `LintSpan`,
  `RuleLevel`, `LintCategory`, `LintRule` trait.
  Four lint rules: L001 (missing observe/act), L002 (act without orient),
  L003 (structural), L004 (policy not checked).
  AST traversal helpers: `walk_definitions`, `walk_expr`, `contains_policy`.
  13 unit tests covering all rules and configuration.
  The CLI binary (`ash-lint` bin) is now a thin wrapper around the library,
  enabling reuse by `ash-lsp-core` (Phase 87) and other consumers.

- Small-step IR compression prototype (TASK-604): added `Stmt`, `Frame`, `Config`,
  and `StmtList` types to `ash-core::small_step` with a lowering function from
  `Workflow`. Implemented an async small-step abstract machine in
  `ash-interp::small_step` (`step` and `run`) that drives configurations to
  completion without recursive big-step descent. Unit tests cover `Done`, `Ret`,
  `Let`, `Seq`, `If`, and `Act` parity with the big-step interpreter.

- Extended small-step IR compression prototype with remaining Workflow variant
  lowerings and error-handling frames (TASK-604 follow-up): added
  `Frame::ForEachIter`, `Frame::Catch`, `Frame::MustGuard`, and
  `Frame::ResumeYield`. Implemented `unwind_stack` for `Maybe` fallback and
  `MustFailure` propagation. Lowered `Observe`, `Orient`, `Propose`, `Decide`,
  `Check`, `With`, `Oblig`, `Maybe`, `Must`, `ForEach`, `Spawn`, `Split`,
  `Kill`, `Pause`, `Resume`, `CheckHealth`, `Yield`, `Set`, `Send`, `Oblige`,
  `CheckObligation`, and `Receive`. Added unit tests for `ForEach` over a list,
  `Maybe` fallback on error, `Must` propagating error as `MustFailure`, and
  `Yield` blocked state. `cargo check` and `cargo clippy` clean.

- Small-step interpreter integration with full runtime context (TASK-604
  follow-up): extended `step` and `run` signatures in
  `ash-interp::small_step` to accept `RuntimeState`, `BehaviourContext`,
  `PolicyEvaluator`, and `StreamContext`. Wired `PolicyEvaluator` into
  `Stmt::Decide`, `BehaviourContext` and `CapabilityPolicyEvaluator` into
  `Stmt::Set`, `StreamContext` into `Stmt::Send`, and `RuntimeState`
  control registry into `Stmt::Kill`, `Pause`, `Resume`, and `CheckHealth`.
  Added `Workflow::Call` variant to `ast::Workflow`, `Stmt::Call` variant
  to `small_step::Stmt`, and corresponding lowering. Added stub match arm
  in big-step `execute_workflow_inner_observed` and small-step `step_inner`.
  Updated all unit tests to pass full runtime contexts.

- LSP diagnostic crate `ash-diagnostic` with `AshLspError` trait, `Severity`,
  `DiagnosticCode`, and `ash_error_to_diagnostic` conversion (TASK-573).
  Implemented `AshLspError` for `ParseError` (E001), `ConstructorError` (E100-E111),
  `TypeEnvError` (E120-E132), `TypeError` (E140-E160), `NameError` (E200-E203),
  `ResolutionError` (E210-E215), and `PurityError` (E300).
  Per-variant diagnostic codes for all error types.
  `TypeError::Obligation` returns `None` from `span()` (no single location).

### Changed

- `ash_error_to_diagnostic` no longer takes a `_source` parameter; the function
  derives the range from the span's line/column fields directly.

- `From<ash_parser::token::Span> for ash_diagnostic::Span` added in `ash-parser`
  with a compile-time size/alignment assertion.  All `AshLspError` impls now
  use `.into()` instead of the manual `to_diag_span` conversion shim.

- Per-variant diagnostic codes for `PurityError` (E300–E304) and `ash_error_to_diagnostic`
  now computes end-position from span byte-width instead of emitting a 1-character range.
  All column/line arithmetic uses saturating subtraction to handle zero-valued spans.

- SPEC-040 §5.4 updated to document the mirrored `Span` approach and the
  actual dependency constraints (ash-diagnostic depends on neither ash-parser
  nor ash-typeck).

- Binding spans for variable references (TASK-570): `Expr::Variable`, `Pattern::Variable`,
  and `PolicyExpr::Var` now carry `{ name, span }` struct variants across surface and core
  ASTs. `ast::Span` derives `Hash` and `Eq` for downstream Salsa usage. All ~400+
  parser/type-checker/interpreter match sites and test constructors updated.

- Comment trivia preservation and `parse_surface_file` API (TASK-571):
  `CommentTable` with `leading`/`trailing` comment capture added to `ParseState`;
  duplicate `skip_whitespace_and_comments` helpers consolidated into
  `crates/ash-parser/src/parse_utils.rs`. New entry points
  `parse_surface_file` / `parse_surface_file_with_path` exposed in `lib.rs`.
  Token helpers auto-classify comments via `set_last_token`.

- Interpreter builtins: `head`, `tail`, `filter`, `map`, `starts_with`, `ends_with`
  (`ash-interp` and `ash-parser`) to support the spec-processor app.

- New `apps/spec_processor` workspace member with initial `.ash` source files
  (`collect.ash`, `types.ash`).

- Design doc: `docs/design/visual-programming/DESIGN-VP-001-MODALITY-ONTOLOGY.md`.

- Parser debug tests for multiline record constructors and closures
  (`fn_parser_tests.rs`) with TODO(TASK-590) annotations on known failures.

### Fixed

- Consolidated duplicate `identifier_with_span` and `is_keyword`
  implementations into `crates/ash-parser/src/parse_utils.rs`.
  All parser modules (`parse_expr`, `parse_pattern`, `parse_policy`,
  `parse_workflow`, `parse_module`) now delegate to the canonical
  implementation, eliminating drift between keyword lists.

- Added source spans to all spanless type-checker error variants
  (TASK-572): `TypeEnvError`, `ConstructorError`, `NameError`,
  `ResolutionError`, and `TypeError` in `ash-typeck` now carry
  `span: ash_parser::token::Span` on every variant. All construction
  sites and tests updated; `Span::default()` used where real spans
  are not yet available.

- Wired `monomorphize_workflow` into the engine pipeline after type checking
  (`Engine::check` now takes `&mut Workflow`) and addressed Phase 83 review
  findings (TASK-564..TASK-568). Fixed missing match arms in
  `monomorphize_expr`, extended `infer_type_from_expr` to handle variables,
  and ensured `cargo clippy --all-targets --all-features` is clean across
  `ash-engine`, `ash-cli`, and `ash-repl`.

- Corrected PLAN-INDEX metadata drift: Phase 70, 78, and 79 marked `Complete`;
  Phase 76 split into `76A` (Complete — runner substrate) and `76B` (Planned —
  synthesis/small-world exploration); TASK-563 status updated to `Complete`.

### Added

- Engine: associated type substitution in monomorphized bodies (TASK-568):
  - `monomorphize_expr` now normalizes `method_info.return_type` and `method_info.params`
    via `TypeEnv::normalize_associated_types` after impl scheme selection
  - Added debug-only `type_contains_associated` assertion to guarantee no
    `Type::Associated` survives monomorphization
  - New integration test: `crates/ash-engine/tests/task_568_monomorphize.rs`

- Type checker: associated types, normalization, and rigid projections (TASK-567):
  - Added `Type::Associated { interface, base, name }` to internal type representation
  - Added `MissingAssociatedType`, `MismatchedProjectionInterface`, and
    `AmbiguousAssociatedType` error variants
  - `register_interface` resolves associated-type projections on interface type params
  - `register_impl` validates associated-type binding completeness and normalizes
    expected return types before body checking
  - `resolve_interface_method_call` normalizes return types after scheme selection
  - Rigid projection rule: identical `Type::Associated` projections unify with empty
    substitution; projections do not unify with arbitrary concrete types

- Engine: post-typecheck monomorphization pass for generic impls (TASK-566):
  - Added `module: Option<Name>` to core `Expr::Call` to preserve interface method calls
  - Added `crates/ash-engine/src/monomorphize.rs` with `monomorphize_workflow`
  - `ImplMethodInfo` now stores lowered core AST method bodies
  - Added `TypeEnv::select_impl_scheme` for public scheme selection
  - Interface method calls in core AST are replaced with concrete impl bodies
  - Fixed `List<T>` lowering inconsistency in `surface_type_to_type`

- Type checker: generic impl schemes, overlap checking, and recursive `where` bound
  resolution (TASK-565):
  - Replaced `HashMap<(String, Type), ImplInfo>` with `Vec<ImplScheme>`
  - Added `OverlappingImpls` and `RecursiveBound` error variants
  - `register_impl` now builds schemes with fresh type variables and checks overlap
    via unification
  - `resolve_interface_method_call` uses ordered scheme search with recursive bound
    checking (depth limit 32)

- `std::regex` interface and Rust backend (TASK-595):
  - Added `std/src/regex.ash` with `find`, `matches`, and `replace` functions
  - Added a Rust regex runtime backend using the `regex` crate
  - Re-exported regex functions from `std/src/lib.ash`
  - Invalid patterns surface clear runtime errors for regex builtins

- Parser/AST support for generic impls, `where` bounds, and associated types (TASK-564):
  - `surface.rs`: `ImplDef` now has `type_params`, `where_bounds`, `associated_type_bindings`
  - `surface.rs`: `InterfaceDef` now has `associated_types`
  - `surface.rs`: `Type::Associated { base, name }` for projections like `S::Ok`
  - `ast.rs`: corresponding core IR fields and `TypeExpr::Associated`
  - Parser: `impl<T> I<T> where T: Bound { type X = Y; ... }` and `interface I { type X; ... }`
  - Lowering: `lower_impl_def`, `lower_interface_def`, `lower_surface_type`

- **Phase 82: Multi-Parameter Interface Methods (SPEC-032)** — Complete implementation across
  parser, AST, type checker, and interpreter (TASK-561 and TASK-562):

  **Parser/AST (TASK-561)**
  - `ImplMethodDef.param: Name` changed to `params: Vec<Name>` in both surface and core AST
  - Interface method signatures now parse `name(Type1, Type2, ...) -> ReturnType`
  - Impl method definitions now parse `name(p1, p2, ...) = expr`
  - `Expr::InterfaceMethodCall` removed from `surface.rs`, `ast.rs`, and `repl/ast.rs`
  - Lowering no longer rejects interface method calls (they lower as ordinary `Expr::Call`)

  **Type Checker / Interpreter (TASK-562)**
  - `resolve_interface_method_call` signature changed from `&Type` to `&[Type]` with zip-unification
  - `register_impl` validates param count and binds each parameter to its declared type
  - `Expr::Call { module: Some(interface_name) }` detects interfaces and routes to multi-param resolution
  - `InterfaceMethodCall` removed from `check_expr.rs`, `lib.rs`, `purity.rs`, `names.rs`,
    `capability_check.rs`, and `eval.rs`
  - All interface calls now route through `Expr::Call`

- **Multi-Parameter Interfaces and Impl Registry Redesign (TASK-563, SPEC-033 §5)** —
  Removed the single type-parameter restriction on interfaces and concrete impl blocks.
  `register_interface` now accepts any number of type parameters; `register_impl` validates
  arity and stores impls keyed by the full interface application (`Pair<Int, String>`)
  rather than a single bare type. `resolve_interface_method_call` constructs the impl head
  from all interface type parameters after unification and reports an error when parameters
  remain underdetermined.

- **Phase 80: First-Class Functions and Closure Values (SPEC-031)** — Complete implementation
  of first-class functions across all nine tasks (TASK-551 through TASK-559):

  **Core IR and Runtime (TASK-551)**
  - `Expr::FnDef { params, return_type, body }` — anonymous function expression in Core IR
  - `Expr::FnApply { func, args }` — user-defined function application (distinct from `Expr::Call`)
  - `Value::Closure { params, body, env }` — closure value capturing `Arc<EnvFrame>` environment
  - `ash_core::env_frame::EnvFrame` — shared environment frame with parent chain for O(1) capture
  - `BindingSlot::Late` — mutex-protected late-binding slot enabling recursive closures
  - `eval_expr` updated: `FnDef` captures current context as `Arc<EnvFrame>`; `FnApply` dispatches to `Value::Closure`
  - `Value::Closure` is `Send + Sync`; serialization intentionally returns an error

  **Lowering (TASK-552)**
  - Built-in function registry distinguishing built-ins (`Expr::Call`) from user closures (`Expr::FnApply`)
  - `lower_fn_def` lowering surface `Expr::FnDef` → Core `CoreExpr::FnDef`
  - Surface `Expr::FnApply` lowered to Core `CoreExpr::FnApply`

  **Type Checker (TASK-553)**
  - `check_expr` handles `Expr::FnDef` → `Type::Fn(params, ret)`
  - `check_expr` handles `Expr::FnApply` → instantiates function type via unifier
  - `Type::Fn(params, ret)` and `Type::Fun(params, ret, effect)` unification rules
  - `Type::Fn` / `Type::Fun` cross-unification explicitly rejected (SPEC-031 §4.8)

  **Engine / Imported Callables (TASK-554)**
  - Imported module-level callables inlined as `Value::Closure` bindings in interpreter context

  **pure_runtime.rs Deletion (TASK-555)**
  - Deleted 476-line `pure_runtime.rs` duplicate interpreter path
  - All previously `pure_runtime`-handled programs now run through single `eval_expr` path
  - Imported callable wiring migrated to closure bindings in `Context`

  **Parser: fn Expressions and Named Local Functions (TASK-556)**
  - `fn(params) [-> Type] { body }` anonymous function expression syntax
  - `fn name(params) [-> Type] { body }` named local function desugars to `let name = fn(...) { ... }`
  - `lower_fn_def` type mismatch fix (`Box<str>` vs `String` in surface AST)

  **Parser: Closure Syntax (TASK-557)**
  - `|params| => expr` sugar for `fn(params) { expr }` — no new AST node, desugars immediately
  - Supports typed params (`|x: Int, y| => x + y`) and empty params (`|| => expr`)
  - `parse_closure_expr` tried first in `expr()` entry point

  **Three-Vertex Boundary Enforcement (TASK-558)**
  - `TypeEnv::workflow_effect: Option<Effect>` — workflow context flag propagated to child scopes
  - `set_workflow_effect(effect)` / `workflow_effect()` API on `TypeEnv`
  - `Expr::FnDef` in pure context → `Type::Fn`; in workflow context → `Type::Fun(…, effect)`
  - `EvalError::BoundaryViolation { value, context }` — runtime variant for escaped closures
  - Fn/Fun unification rejection already enforced in `unify()` (pre-existing, now tested)

  **End-to-End Validation (TASK-559)**
  - SPEC-031 §13.1 conformance integration tests in `ash-interp/src/eval.rs`:
    `task559_fndef_produces_value_closure`, `task559_fnapply_calls_closure`,
    `task559_closure_captures_enclosing_scope`, `task559_higher_order_function_apply`,
    `task559_recursive_closure_via_late_binding`, `task559_closure_is_send_sync`,
    `task559_closure_serialization_returns_error`, `task559_fnapply_non_callable_returns_error`,
    `task559_fnapply_wrong_arity_returns_error`
  - `cargo test --all`: 0 failures across all crates

### Fixed

- Phase 80 code review follow-up: fixed `String` vs `Box<str>` compilation errors in three `check_expr.rs` test functions (`task558_fndef_annotated_param_constrains_inference`, `task558_fndef_annotated_return_type_verified` matching and conflicting cases). `Name` is `Box<str>`; tests were using `.to_string()` instead of `.into()`.
- Added escape case 2 test: `task558_escape_case_2_store_fun_in_state_rejected` verifies `Type::Fun` does not unify with `Type::Fn`, preventing storing effectful closures in pure state fields.
- Added `task559_boundary_violation_on_context_boundary_crossing` test demonstrating `EvalError::BoundaryViolation` construction and message.
- Added `task559_module_level_fndef_never_produces_closure` test: module-level functions return their result directly (never `Value::Closure`), contrasted with expression-level `FnDef` which does produce closures.
- Tracked follow-up TASK-560: `annotation_name_to_type` silently falls back to fresh type variables for unknown type names (user-defined types).
- **TASK-560:** Replaced `annotation_name_to_type` with TypeEnv-aware `annotation_to_type` resolver. Unknown type annotations in `Expr::FnDef` parameters and return types now produce `ConstructorError::UnknownTypeAnnotation` errors instead of silently falling back to fresh type variables. User-defined types registered in `TypeEnv` resolve to `Type::Constructor`. Three new conformance tests.
- Added memory-leak note to SPEC-031 §4.6: recursive closures via `BindingSlot::Late` form reference cycles through `Arc<EnvFrame>` and are not reclaimed until the enclosing workflow is dropped. Acceptable for short-lived CLI usage or bounded tests, but not for long-running engines.
- **PLAN-029 / Phase 82:** Multi-Parameter Interface Methods — planned from SPEC-032. Tasks TASK-561 and TASK-562.
- **PLAN-030 / Phase 83:** Multi-Parameter Interfaces, Generic Implementations, and Associated Types — planned from SPEC-033, SPEC-034, and SPEC-035. Tasks TASK-563 through TASK-568.

- Resolved all build errors and clippy warnings introduced in commit 09143dd (TASK-556 parser work) and pre-existing in ash-engine. Fixes include: unused import in `llm_e2e_usability_tests.rs`, needless borrow in `ash-interp/src/eval.rs`, `#[ignore]` without reason in `execute.rs`, clone-on-copy and single-match-else in `module_loader.rs`, collapsible-if and collapsible-match in `chat.rs`, casting and doc-markdown issues in `embeddings.rs`, too-many-lines in `provider.rs`, needless-pass-by-value/map-or/box-default/manual-string-new/doc-markdown in `stream_adapter.rs` and `stream_storage.rs`, PartialEq-without-Eq and doc-markdown in `tool_dispatch.rs`, used-underscore-binding/collapsible-if/option-if-let-else/doc-markdown in `lib.rs`, and test-code cleanups in `llm_integration_tests.rs`, `llm_engine_integration.rs`, and `ast.rs`.

### Added

- **SPEC-031: First-Class Functions and Closure Values** — Plan for Phase 80:
  - SPEC-031 v0.4 (approved): `fn(params) { body }` as expression producing `Value::Closure`, named local fn desugars to `let name = fn(...)`, `|x| => body` closure syntax, `Arc<EnvFrame>` shared scope capture, `BindingSlot::Late` for recursion, `Expr::FnApply` for user calls, `Type::Fn`/`Type::Fun` three-vertex enforcement.
  - PLAN-028: 9 tasks (TASK-551 through TASK-559), 5 migration phases (A-E), deletes 476 lines of `pure_runtime.rs`.
  - Phase 80 registered in PLAN-INDEX.

### Added

- **Phase 78: Module Type Resolution (SPEC-030)** — Two-pass type collection, module-file checking, and pub fn diagnostics:
  - Two-pass type registration with pre-declaration in `TypeEnv` for forward references (TASK-539). Extracted `is_placeholder` helper for deduplicated placeholder detection.
  - `pub mod <name>;` child module loading in `collect_module_exports` (TASK-540). Recursively loads child exports into `child_modules` field without flattening into parent.
  - `Engine::check_module_file()` API for validating non-workflow module files (TASK-541). CLI `ash check` detects module files and reports type/fn counts.
  - `PubFnDiagnostic` warning type for unparseable `pub fn` snippets (TASK-542). `parse_supported_pub_fn_callable` returns `Result` instead of silent `Option`. Diagnostics surfaced via `check_module_file`.
  - `ModuleFileCheckResult` public struct with type count, fn count, warnings, and errors.
  - Conformance tests ST-6 through ST-13 for SPEC-030 §4-5.
  - LLM stdlib end-to-end validation (TASK-543). Structural tests replacing string-matching: type name verification via `collect_public_type_defs_from_source`, pub fn parse coverage via `count_pub_fn_snippets`, import path resolution, and cross-cutting stdlib file validation.
  - **Key finding**: 16 of 23 `pub fn` in prompt.ash use record constructors unsupported by `parse_fn_definition`, causing silent export dropping. Documented via `#[ignore]` target test.

- Fix 2-segment `use` path resolution and improve import error context (TASK-547):
  - `collect_module_exports` now gracefully skips workflow parse failures in child modules (e.g. `dispatch.ash`), preventing them from killing the entire module's re-export collection. Mirrors the existing `pub fn` graceful-skip pattern.
  - `merge_use_exports` silently skips re-exported items not yet defined in the target module, allowing `mod.ash` files to reference forward-declared types and functions.
  - Improved error messages: `pub use` parse errors now include the module file path; `resolve_use_target` includes the search root; import parse errors include the original import text. Replaces opaque "ContextError" with actionable context.
  - Regression tests: `use llm::Role` and `use llm::Message` resolve via `mod.ash` re-exports; `use nonexistent::Foo` produces "not found" error.

- Add missing SPEC-029 prompt functions and fix `has_tool_calls` signature (TASK-548):
  - `append_response(messages, response)`: appends assistant message from `ChatResponse` to conversation history.
  - `append_tool_result(messages, call_id, content)`: appends tool result message to history.
  - `is_final(response)`: checks if `finish_reason` is `"stop"` or `"length"`.
  - `render_template(template, vars)`: stub for template variable substitution (awaiting runtime `string::replace`; `vars` is `Map<String, String>` alias for `List<(String, String)>`).
  - New stdlib type `Map<K, V>` in `std/src/map.ash` -- generic alias for `List<(K, V)>`.
  - `has_tool_calls` signature fixed from `(msg: Message)` to `(response: ChatResponse)` per SPEC-029 §4.2.3.
  - `mod.ash` re-exports updated for all four new functions.
  - Total `pub fn` count in `prompt.ash`: 23 → 27; parseable count: 12 → 15.

- Fix three-vertex violations in orchestration modules (TASK-549):
  - `router.ash`: split `fn classify_route` into pure `fn build_classify_message` + `fn parse_route`; moved `complete()` call into `workflow router` body.
  - `supervised.ash`: split `fn request_approval` into pure `fn build_approval_message` + `fn parse_supervisor_response`; moved `complete()` call into `workflow supervised_agent` body.
  - No `fn` in either file now references a dispatch workflow. Three-vertex compliance tests added.

- Rename `Message` field `role` to `sender` to avoid Ash keyword collision (TASK-549 follow-up):
  - `role` is a reserved keyword in Ash's governance model; using it as a struct field name, parameter name, or function name caused the parser to reject 12 of 27 `pub fn` in prompt.ash.
  - Field renamed across `types.ash`, `prompt.ash`, `mod.ash`, and Rust provider code (`chat.rs`, `tool_dispatch.rs`).
  - Function `role(msg)` renamed to `sender(msg)`, helper `role_name` renamed to `sender_name`.
  - `mod.ash` re-export updated: `role` -> `sender`.
  - Parseable pub fn count: 15 -> 24 of 27 (9 functions unblocked by removing keyword collision).

- End-to-end validation of LLM stdlib usability (TASK-550):
  - All 27/27 `pub fn` in prompt.ash parse cleanly through the engine.
  - `use llm::Role`, `use llm::Message`, `use llm::ChatResponse` all resolve from application code.
  - `ash check` reports 0 errors/warnings on all llm/ files.
  - Three-vertex compliance: no `fn` in router.ash or supervised.ash calls dispatch workflows.
  - SPEC-029 section coverage audit: all 11 types, constructors, inspectors, renderers, and agent workflows verified.
  - End-to-end workflow parsing test: `.ash` file constructing `Message` values with `sender`/`content` fields parses through the full engine pipeline.
  - PLAN-027 complete.

- **Phase 77: LLM Standard Library** — Complete LLM capability implementation for the Ash language:
  - LLM provider module with async-openai integration (TASK-516). Adds `async-openai` dependency for OpenAI-compatible HTTP communication.
  - `LlmConfig` struct for per-provider connection settings with validation, defaults, and API key redaction (TASK-517).
  - `LlmProvider` capability provider with multi-provider routing, lazy client creation, and list_models action (TASK-518).
  - Chat completion actions (`chat`, `chat_with_tools`) with message conversion, tool definition support, parameter validation, and error mapping (TASK-519).
  - Integration tests with wiremock for LLM provider error mapping (TASK-519).
  - Streaming adapter for chat responses with SSE chunk parsing (TASK-520). Implements `ChatChunk` and `ToolCallDelta` types per SPEC-029 §3.
  - Stream error propagation tests verifying `pull_stream_chunk` returns `ExecutionFailed` on upstream failures per SPEC-029 §9.4 SC4 (TASK-520).
  - Tool dispatch helpers for converting between Ash Values and OpenAI tool formats (TASK-521). Includes `ToolCall` extraction and tool result formatting.
  - Embeddings action with postcondition verification (TASK-522). Supports `text-embedding-3-small` and similar models with `Embedding` return type.
  - Ash stdlib types in `std/src/llm/types.ash`: `Role`, `Message`, `ToolCall`, `ToolCallDelta`, `ToolDef`, `ChatResponse`, `Embedding`, `ChatChunk`, `Usage`, `ChatOptions` (TASK-524-525).
  - Prompt constructors in `std/src/llm/prompt.ash`: `system`, `user`, `assistant`, `assistant_with_tools`, `tool_result` (TASK-526).
  - Prompt inspectors in `std/src/llm/prompt.ash`: `is_system`, `is_user`, `is_assistant`, `is_tool`, `role`, `content`, `get_tool_calls`, `has_tool_calls` (TASK-527).
  - Prompt renderers in `std/src/llm/prompt.ash`: `render_plaintext`, `render_markdown` for conversation formatting (TASK-528).
  - OpenAI capability declaration in `std/src/llm/openai.ash` with `Llm` capability and action signatures per SPEC-029 §5 (TASK-529).
  - Dispatch workflows in `std/src/llm/dispatch.ash`: `complete`, `complete_with_tools`, `complete_tuned`, `ask`, `stream`, `embed`, `list_models` (TASK-530).
  - Loading workflows in `std/src/llm/loading.ash`: `load_prompt`, `load_system_prompt` for prompt file loading (TASK-531).
  - Agent orchestration workflows: `conversation` (TASK-532), `tool_agent` (TASK-533), `router` (TASK-534), `supervised_agent` (TASK-535).
  - Comprehensive integration tests in `crates/ash-engine/tests/llm_integration_tests.rs` with mock backends covering chat, tools, streaming, embeddings, error handling, and multi-provider routing (TASK-536).
  - Engine-level integration tests using `with_llm_capabilities()` builder and `execute_core_workflow()` to verify engine → LLM provider dispatch for chat, list_models, embed, and result binding (TASK-523).
  - Corrected `LlmProvider` effect from `Deliberative` to `Operational` so Act dispatch through `CapabilityContext` succeeds (TASK-523).
  - `Engine::execute_core_workflow()` test helper for executing hand-constructed core IR through the engine's registered capability providers (TASK-523).
  - Module-level documentation in `std/src/llm/mod.ash` with overview, quick start example, and architecture documentation (TASK-538).
  - Stdlib verification tests in `crates/ash-engine/tests/llm_stdlib_tests.rs` (16 tests) validating types.ash has all 11 SPEC-029 types, prompt.ash has constructors (TASK-526), inspectors (TASK-527), and renderers (TASK-528), and all .ash files are valid UTF-8 (TASK-524/525).
  - Fixed `ash-cli` `value_to_json` exhaustiveness for `Value::Float` and `Value::Stream` variants.
- Drafted [DESIGN-024: Property Generation Substrate](docs/design/DESIGN-024-PROPERTY-GENERATION-SUBSTRATE.md), defining the canonical generated-case model, bounded value-domain substrate, deterministic seed-driven generation pipeline, and staged implementation order needed to move Ash property testing beyond bounded reruns into true generated-input execution.
- Drafted [DESIGN-022: Synthesized Contract / Policy / Obligation Cases](docs/design/DESIGN-022-SYNTHESIZED-CONTRACT-POLICY-OBLIGATION-CASES.md), defining the stable introspection, executable case model, oracle model, and staged implementation order needed to turn Phase 76 synthesized test planning into real executable synthesized cases.
- Drafted [DESIGN-023: Small-World Exploration Substrate](docs/design/DESIGN-023-SMALL-WORLD-EXPLORATION-SUBSTRATE.md), defining the canonical world model, finite-domain enumeration substrate, oracle model, and staged implementation order needed to move small-world testing beyond bounded reruns into true world exploration.
- **Phase 76: Ash Test Runner V1 (substantial landing, phase still open)** — Added a CLI-integrated Ash test runner with:
  - `ash test` command surface, human/JSON output, and source-scoped synthesized selection (`contracts`, `policies`, `obligations`)
  - per-test panic capture, isolated execution, and timeout containment without aborting the suite
  - authored test discovery from conventional roots plus direct kind-directory/file execution
  - `-- @test` file-header metadata parsing for names, tags, timeout, xfail, seed, max_cases, and max_worlds
  - a minimal exported `std::test` assertion surface usable from authored Ash tests
  - bounded property and small-world execution routing with seed/case/world reporting
  - opt-in synthesized test planning from contracts, policies, and obligations with explicit authored-vs-synthesized labeling
  - explicit deferred follow-up items recording that true synthesized execution and true generative/small-world exploration will be developed after spec work improvement

### Fixed

- **Phase 76 remediation**: closed the earlier runner gaps by implementing explicit synthesized-source selection, fixing `--only-synthesized` to exclude authored tests, enabling direct kind-directory discovery, aligning authored metadata parsing with documented `-- @test` syntax, making the minimal `std::test` surface usable from authored tests, preventing bogus property/small-world metadata from leaking onto ordinary tests, and wiring bounded property/small-world execution into the suite path.
- Pure-functions closeout verification gaps: `ash check` now rejects undefined pure-function calls with an unknown-function diagnostic, rejects capability targets used with `module::name(...)` pure-call syntax with a wrong-target capability diagnostic, uses explicit capability-symbol registration instead of the previous name-shape heuristic for qualified pure-call wrong-target detection, and the engine check path consistently runs workflow-definition validation instead of the older shallow workflow-only check for ordinary files.
- Pure-functions phase bookkeeping is now aligned with the verified repository state: PLAN-023 is marked complete, Phase 75 in PLAN-INDEX is marked complete, and the remaining pure-functions task records no longer show stale planned status.

### Added

- Drafted [DESIGN-021: Ash Test Runner V1](docs/design/DESIGN-021-ASH-TEST-RUNNER-V1.md), defining a fail-contained `ash test` runner integrated with the CLI, a dedicated Ash test library phase for assertions/helpers, v1 support for unit/integration/e2e/property/small-world execution, explicit authored vs synthesized test labeling, and contracts/policies/obligations as opt-in metadata sources for synthesized tests together with recommended test metadata structure in the codebase.
- Planned [PLAN-024: Ash Test Runner V1](docs/plan/PLAN-024-ASH-TEST-RUNNER-V1.md), added Phase 76 to [PLAN-INDEX](docs/plan/PLAN-INDEX.md), and authored [TASK-509](docs/plan/tasks/TASK-509-ash-test-runner-substrate.md) through [TASK-515](docs/plan/tasks/TASK-515-ash-test-runner-docs-and-phase-verification.md) to land the runner substrate, Ash test library surface, authored test metadata/discovery model, synthesized tests from contracts/policies/obligations, bounded property/small-world execution, and final verification/bookkeeping.
- Phase pure-functions closeout progress: TASK-506 and TASK-507 are now marked passed in the plan/task tracker. The stdlib pure-function surface was aligned to `Fn(...) -> ...`, stdlib/parser/module-resolution conformance coverage was expanded for imported and qualified pure function calls, and engine/runtime integration now preserves pure-runtime routing for local fn programs without forcing unsupported lowering of pure-only fn bodies.

- Pure-functions follow-up docs pass: updated [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-009](docs/spec/SPEC-009-MODULES.md), and [SPEC-012](docs/spec/SPEC-012-IMPORTS.md) to align on the explicit capability-call baseline (`provider:action(...)` is the capability invocation form; `module::symbol` remains module qualification / symbol resolution metadata and does not become an alternate call surface), updated [SPEC-022](docs/spec/SPEC-022-WORKFLOW-TYPING.md) examples to use that same baseline, updated [DESIGN-020](docs/design/DESIGN-020-PURE-FUNCTIONS-THREE-VERTEX-MODEL.md) to mark `panic` as resolved/frozen for this phase, aligned [SPEC-027](docs/spec/SPEC-027-PURE-FUNCTIONS.md) and [PLAN-023](docs/plan/PLAN-023-PURE-FUNCTIONS-PHASE.md) with the frozen [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md) `Type::Fn(Vec<Type>, Box<Type>)` shape and effect-neutral fn-call wording, and refreshed this changelog entry to match the actual scope of the follow-up.

- TASK-493: Frozen Stdlib IO V1 Contract. Updated SPEC-009-MODULES.md, SPEC-012-IMPORTS.md, SPEC-017-CAPABILITY-INTEGRATION.md, SPEC-010-EMBEDDING.md, and 2026-04-10-stdlib-io-v1-design.md to document the canonical `io` namespace, v1 module tree, capability boundary, and canonical import style.
- TASK-494: Added io root and pure path surface. Created `std/src/io/mod.ash` with Error, ErrorKind, and Result<T> types. Created `std/src/io/path.ash` with PathBuf type and pure path functions. Updated lib.ash with io exports. All 24 parser tests pass.
- TASK-495: Added io::stdio surface and provider alignment. Created `std/src/io/stdio.ash` with Stdio capability and functions. Aligned with existing StdioProvider. All 17 tests pass.
- TASK-496: Added io::fs, io::dir, io::meta surface and expanded FsProvider. Created fs.ash with file operations, dir.ash with directory operations, meta.ash with metadata operations. Expanded FsProvider with 11 new actions. 176 tests pass.
- TASK-497: Added io::buf buffered helpers. Created `std/src/io/buf.ash` with read_to_end, read_to_string, write_all, and lines functions. All tests pass.
- TASK-498: Bootstrap io modules through runtime wiring. Created io_stdlib_wiring_test.rs with 16 tests. Added provider wiring tests for io capabilities. All 25 tests pass.
- TASK-499: Added integration tests and examples. Created examples/03-io/ with 3 example workflows. Created tests/std/io_*.ash with 31 test fixtures. All tests pass.
- TASK-500: Final docs and verification for Phase 74. cargo fmt clean. cargo check passes. Fixed pre-existing clippy warnings. 172 IO-specific tests pass. Pre-existing test failures identified and distinguished from Phase 74 work.

### Fixed

- Fixed `let <name> = <cap-call>` sugar boundary check consuming newlines and line comments (Phase 73 regression). Added `skip_horizontal_ws_and_comments` that preserves newlines as statement delimiters. Fixed `lower_stmts_to_nested` rfold overwriting explicit `act ... then` continuation bodies — existing continuations now compose with the outer tail via `Seq`. Updated TASK-486 through TASK-492 status from Planned to Done.
- Fixed `ash-parser` capability definition property generators to validate identifiers through the parser's real `identifier` acceptance path instead of a stale duplicated keyword list. This removes false proptest failures on reserved words such as `do`.

### Added

- Planned Phase 74 as the stdlib `io` v1 implementation phase. Added [Stdlib `io` V1 Design](docs/plans/2026-04-10-stdlib-io-v1-design.md), [Stdlib IO V1 Implementation Plan](docs/plans/2026-04-10-stdlib-io-v1-implementation-plan.md), [PLAN-022](docs/plan/PLAN-022-STDLIB-IO-V1.md), and [TASK-493](docs/plan/tasks/TASK-493-freeze-stdlib-io-contract.md) through [TASK-500](docs/plan/tasks/TASK-500-stdlib-io-docs-and-verification.md) to land the first top-level `io` stdlib family with pure path values, capability-backed stdio/filesystem modules, provider/runtime wiring, and end-to-end examples.

- Planned Phase 72 as the focused closeout phase for the remaining Phase 71 architectural gap. Added [DESIGN-018](docs/design/DESIGN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md), [PLAN-018](docs/plan/PLAN-018-MODULE-SCOPED-CAPABILITY-RESOLUTION-CLOSURE.md), and [TASK-480](docs/plan/tasks/TASK-480-module-scoped-resolution-api.md) through [TASK-484](docs/plan/tasks/TASK-484-phase-71-closeout-docs-and-verification.md) to finish module-scoped shared-context resolution and remove the last type-checker fallback path.

- **Phase 73: Action Result Binding and Continuation** — Extended `Workflow::Act` with `result_name: Option<Name>` and `continuation: Box<Workflow>` so capability actions can produce values that flow back into the workflow. Three new surface forms: `act ... then <workflow>` (discard result, continue), `act ... as <name>` (bind result, lexical-scope continuation), and `let <name> = <cap-call>` sugar (parse-time recognition in `let_stmt()`). Core, surface, lowering, parser, interpreter, and typeck all updated. 1632 tests green. See [DESIGN-019](docs/design/DESIGN-019-ACTION-RESULT-BINDING.md), [PLAN-019](docs/plan/PLAN-019-ACTION-RESULT-BINDING.md), [TASK-486](docs/plan/tasks/TASK-486-core-act-continuation-shape.md) through [TASK-492](docs/plan/tasks/TASK-492-act-continuation-docs-and-verification.md).

- **Phase 71: Module-Owned Capability Resolution** - ✅ **COMPLETE**. Symbolic capability calls resolve from module/import-owned metadata. **Key deliverables:** (1) `CapabilityExport` and `CapabilityResolutionContext` types; (2) `CapabilityPipeline` integrates module exports with import resolution; (3) `LoweringContext` for capability-aware lowering; (4) Bridge `with_builtin_mappings()` **REMOVED** from parser and typeck; (5) Import resolution properly scoped by `ModuleId`; (6) Lowering and type checking share authoritative resolution context. Phase 71 completed via Phase 72 closure.

- **Phase 72: Module-Scoped Capability Resolution Closure** - ✅ **COMPLETE**. Closed the architectural gap in Phase 71. **Key deliverables:** (1) `CapabilityResolutionContext::resolve_unqualified(current_module, name)` API requires explicit `ModuleId`; (2) `CapabilityResolutionContext::resolve_qualified_to_strings(module_name, capability_name)` for dedicated qualified resolution; (3) Removed module-agnostic `resolve_for_lowering()` global search; (4) Lowering threads `ModuleId` through `LoweringContext::with_capability_context_for_module()`; (5) Type checking threads `ModuleId` through `CapabilityChecker::with_resolution_context_for_module()`; (6) Qualified capability calls (`module::capability(...)`) use dedicated qualified resolution API, not string-building fallback; (7) **REMOVED** `CapabilityChecker` fallback resolver - capability checking now relies solely on shared `CapabilityResolutionContext`; (8) Verified: 525 ash-parser tests pass, 532 ash-typeck tests pass. **NOTE:** `NameResolver` in `ash-typeck/src/names.rs` retains a `CapabilityResolver` for non-symbolic resolution purposes; 5 ash-engine conditional-execution tests fail (pre-existing interpreter issues, unrelated to capability resolution).

- Planned Phase 71 as the follow-on resolver integration phase for module-owned symbolic capability resolution. Added [DESIGN-017](docs/design/DESIGN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md), [PLAN-017](docs/plan/PLAN-017-MODULE-OWNED-CAPABILITY-RESOLUTION.md), dated planning/design handoff docs in `docs/plans/`, and authored [TASK-471](docs/plan/tasks/TASK-471-spec-module-owned-capability-resolution.md) through [TASK-479](docs/plan/tasks/TASK-479-module-owned-capability-resolution-verification.md) to replace the Phase 70 bridge resolver with module/import-owned capability metadata.

### Changed

- Reframed Phase 70 as an in-progress bridge implementation rather than a completed final resolver architecture. Active docs now distinguish the landed split-dispatch/runtime surface work from the still-open module-system integration needed for symbolic capability resolution.

- **Phase 69: Unified Action System** - Completed full migration (TASK-449 through TASK-462). Key changes: (1) `Action.arguments` changed from `Vec<Expr>` to `Vec<Value>` with eager evaluation at ACT execution boundary; (2) New unified `CapabilityProvider` trait in `ash_core::capability` with `observe(&[Constraint])` and `execute(&Action)` methods; (3) New unified `CapabilityError` enum replacing split error types; (4) All providers (FsProvider, StdioProvider, McpProvider) migrated to unified trait; (5) `InterpProviderAdapter` removed - providers now use unified trait directly; (6) Engine builder and RuntimeState updated to use unified trait throughout; (7) CLI RuntimeArgProvider migrated; (8) All integration tests updated and passing; (9) Full clippy clean with strict warnings. This is a breaking change that removes the old engine-side `CapabilityProvider` trait and `ProviderError` type.

- Planned Phase 69 as the Unified Action System migration. Added [PLAN-015](docs/plan/PLAN-015-UNIFIED-ACTION-SYSTEM.md), corrected it so parser/lowering and interpreter ACT evaluation land in the same first phase as the `Action` representation change, and authored the follow-on task records [TASK-451](docs/plan/tasks/TASK-451-capability-context-unified-trait.md) through [TASK-462](docs/plan/tasks/TASK-462-final-integration-testing.md) so the later interpreter, engine-provider, error-handling, documentation, and integration-testing work is decomposed into executable steps.

- **Phase 68: Surface Binding Scope Conformance** - Completed all tasks (TASK-443 through TASK-447) establishing a canonical lexical-scope contract for newline-separated surface statements. The phase removes ambiguity around statement list scoping by making lexical-block lowering normative and aligning parser, lowering, type checking, interpreter, and CLI conformance tests to one continuation-owned scope model. Core achievements: (1) SPEC-002/SPEC-003/SPEC-004/SPEC-025 amendments establish that surface statement lists lower canonically to nested `LET ... in cont` structures with `SEQ` reserved for non-binding sequencing; (2) Parser and lowering normalize statement lists into the canonical lexical-block form; (3) Type checker aligns with lexical-block lowering so earlier bindings are visible to later statements; (4) Interpreter executes faithfully to the canonical lowered form with correct terminal statement handling; (5) End-to-end conformance tests confirm `ash check`, `ash run`, and `ash trace` agree on lexical block scope. The phase deliverable is one unambiguous lexical-scope contract backed by normative spec text and aligned implementation across all phases.

- Completed TASK-443 as a spec-only pass freezing the normative surface-to-core scoping rule. [docs/spec/SPEC-002-SURFACE.md](docs/spec/SPEC-002-SURFACE.md) now defines the canonical lowering rule for newline-separated statement lists to nested `LET ... in cont` forms, establishing lexical scoping where earlier bindings are visible in later statements. [docs/spec/SPEC-003-TYPE-SYSTEM.md](docs/spec/SPEC-003-TYPE-SYSTEM.md) documents the type-environment consequence, while [docs/spec/SPEC-004-SEMANTICS.md](docs/spec/SPEC-004-SEMANTICS.md) and [docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) now explicitly state they operate over the canonical lowered form. This removes the previous ambiguity around whether statement lists lower to `LET` versus `SEQ` and establishes one coherent lexical-scope contract across all four specs.

- Planned Phase 68 as a spec-first surface binding scope conformance phase. The repo now includes a dedicated design/implementation plan plus TASK-443 through TASK-447 to remove the ambiguity around newline-separated statement scope by making lexical-block lowering normative in `docs/spec` and then aligning parser, lowering, type checking, interpreter behavior, and CLI-facing conformance coverage to that one model.

- Completed TASK-442 by making ordinary file workflows resolver-backed across local modules, `ASH_LIBRARY_PATH` library roots, and the built-in stdlib. `ash-engine` now resolves multi-file user modules from the workflow tree, supports version-qualified roots such as `math@1::vector`, loads imported stdlib/user `pub type` definitions during ordinary file execution, and inlines the current supported callable subset for imported local helper workflows, stdlib `pub fn` helpers, and `pub use` re-exports such as `prelude::{is_some}`.

- Completed TASK-441 by switching the repository GitHub Actions workflows to manual dispatch only. [.github/workflows/ci-fast.yml](.github/workflows/ci-fast.yml), [.github/workflows/differential-testing.yml](.github/workflows/differential-testing.yml), and [.github/workflows/lean-reference.yml](.github/workflows/lean-reference.yml) now use `workflow_dispatch` as their only trigger, disabling automatic `push`, `pull_request`, and scheduled CI runs while preserving manual execution from the Actions UI/API.

- Completed TASK-436 as a docs/reference/planning contract pass for retained completion parity. The repo now includes [docs/reference/retained-completion-parity-contract.md](docs/reference/retained-completion-parity-contract.md), which freezes the exact boundary between full semantic `CompletionPayload` parity, conservative retained-completion summaries, terminal-visible subset-only retained slices, and dimensions that remain outside retained-completion parity itself. [docs/reference/semantic-execution-record-contract.md](docs/reference/semantic-execution-record-contract.md), [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md), and follow-on task surfaces now cite that contract directly so later retained-completion work can extend fidelity slice-by-slice without conflating retained observation with the broader execution-record contract.

- Completed TASK-438 as the canonical conformance corpus/result-format definition pass for Phase 67. The repo now includes [docs/reference/canonical-ir-semantics-corpus.md](docs/reference/canonical-ir-semantics-corpus.md) and [docs/reference/canonical-semantics-result-format.md](docs/reference/canonical-semantics-result-format.md), freezing one shared canonical IR case inventory, one file-backed corpus layout, one machine-readable expected-result envelope for exact versus allowed-set comparisons, and one explicit bounded-nondeterminism policy for `Par` and `receive` cases. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md), [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md), and downstream TASK-439/TASK-440 surfaces now align around that shared corpus/result substrate.

### Fixed

- Fixed documentation in SPEC-013-STREAMS.md to mark the `par` examples as historical. The "With Parallel Composition" section (10.1) is now marked as "(Historical)" with an explanatory note that `par` is no longer part of the active language contract, preventing confusion with current syntax.

- Fixed name resolver to restore duplicate pattern binding rejection while allowing shadowing across statements. The name resolver now correctly distinguishes between pattern-level bindings (which must be unique within a single pattern) and statement-level bindings (which may shadow earlier bindings). This restores the TASK-005 invariant that patterns cannot contain duplicate binders, as documented in `docs/plan/tasks/TASK-005-patterns.md`. The fix introduces a `pattern_bindings` set to track bindings within the currently-processed pattern and rejects duplicates with a `DuplicateBinding` error, while the existing `bind()` method continues to allow shadowing for statement-level bindings. Regression test coverage added in `crates/ash-typeck/tests/pattern_duplicate_bindings.rs`.

- Fixed conformance mismatch between parser and typechecker for propose binding. The parser already treated `propose ... as x` as a lexical-binding statement (per Phase 68 surface-binding contract), but the typechecker was rejecting all `Workflow::Propose { binding: Some(_) }` as unsupported MVP behavior. The typechecker now accepts propose bindings and binds them with a fresh type variable (consistent with how observe bindings work) until full result semantics are implemented. This aligns the typechecker with the parser's behavior and resolves the Phase 68 conformance violation where code that parsed correctly would fail type checking.

- Fixed parser conformance tests in `ash-parser` to align with terminal statement optimization. The `lexical_block_scope.rs` tests now expect bare `Ret`/`Done` statements instead of `Seq(ret, Done)` for terminal statements, which is the correct canonical form that ensures proper runtime behavior (see SPEC-025 SEQ-ADVANCE rule).

- Fixed clippy warnings for unused Par-related code in `ash-interp`. Added `#[allow(dead_code)]` to historical parallel execution helper functions in `execution_record.rs` (including `merge_parallel`, `ParallelTraceEvent`, `trace_event_timestamp`, `join_parallel_provenance`, `merge_parallel_traces`, `merge_parallel_success`, `merge_parallel_rejection`, `merge_parallel_terminal`, and `ExecutionRecorder::replace_with_snapshot`) and to test helper `test_role_with_obligation` in `execute.rs`. These functions are retained for documentation/reference purposes following the Par corpus removal in Task 5. Also fixed unused import and variable warnings in `par_removal_tests.rs`. All workspace verification passes: `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test -p ash-interp --test par_removal_tests`, and `cargo doc --no-deps`.

- **Phase 68: Surface Binding Scope Conformance Fixes** - Fixed parser terminal statement handling as part of TASK-447 completion. The parser's `lower_stmts_to_nested` function now correctly identifies terminal statements (`Ret`, `Done`, and `Act`) and avoids wrapping them in unnecessary `Seq` constructs when the continuation is `Done`. This ensures that workflows like `workflow main { ret 42 }` and `workflow main { let x = 10; ret x }` return their actual values instead of `null`. Fixed API usage in `ash-engine` lexical scope tests (TASK-446 follow-up). Tests now correctly use `engine.execute(&workflow)` and `engine.execute_with_input(&workflow, input)` instead of the incorrect `workflow.execute()` pattern. Tests compile successfully and properly exercise the full parsing/typechecking/execution pipeline for lexical scope functionality.

- Completed TASK-437 in `ash-interp` as one narrow retained-completion parity slice: child-owned retained completions now preserve exact `CompletionPayload.effects` parity from the authoritative sealed child execution record instead of workflow-form conservative upper bounds. The retained effect carrier still remains bounded to terminal/reached effect contents only, control tombstones still keep `effects: None`, and retained obligations/provenance remain on their existing honest subset/conservative classifications.

- Completed TASK-435 in `ash-interp` as the first runtime-side `Par` aggregation realization against the frozen TASK-434 contract. Spawned child executions no longer overwrite `RuntimeState::last_execution_record()`, and `Par` execution now preserves branch-local execution records per branch before rebuilding the enclosing parent record from aggregated trace, effect, obligation, and provenance snapshots. Focused regression coverage now includes top-level/stream authority preservation after spawn and branch-local carrier aggregation for `Par`.

- Completed TASK-434 as a docs/spec/reference/planning contract pass for `Par` branch-state and helper-backed aggregation. [docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) now freezes one explicit `Par` branch-local carrier contract: live `ParState(bs)` evaluation owns branch-local `Γ`, `Ω`, `π`, `T`, `ε̂`, and branch terminal payloads; helper-backed aggregation is defined explicitly for all-success completion, mixed success/rejection terminal sets, and blocked/nonterminal branch collections; and implementation conformance is stated modulo admitted branch interleaving and helper-owned concurrent aggregation latitude rather than presentation order. [docs/reference/semantic-execution-record-contract.md](docs/reference/semantic-execution-record-contract.md), [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md), [docs/ideas/IMPLEMENTABILITY-REPORT.md](docs/ideas/IMPLEMENTABILITY-REPORT.md), and the TASK-434 record now align around that frozen contract so TASK-435 can implement runtime aggregation directly without re-deriving the `Par` semantics from MCE prose.

- Completed TASK-433 in `ash-interp` as the first authoritative execution-record substrate slice. The interpreter now owns an explicit `ExecutionRecord` / `ExecutionRecorder` runtime carrier for execution phase, obligations, provenance, cumulative trace, and cumulative effect summary; top-level behaviour/stream execution paths snapshot that record into `RuntimeState::last_execution_record()`; and direct semantic terminal projection is now exposed through `project_workflow_outcome()` and `project_completion()`. Focused regression coverage now includes terminal success projection, terminal rejection projection, and cumulative orient trace/effect carriage, while the surrounding planning/runtime-cleanup corpus records this as a first carrier-packaging slice rather than full `Par` aggregation or retained-completion parity closure.

- Completed TASK-432 as a docs/reference/planning contract pass for cumulative semantic carrier alignment. The repo now includes [docs/reference/semantic-execution-record-contract.md](docs/reference/semantic-execution-record-contract.md), which freezes the canonical runtime-facing semantic execution record for cumulative `Ω`, `π`, `T`, and `ε̂` together with an explicit runtime-facing phase taxonomy (`Running`, `Blocked(...)`, terminal success/rejection, and `Invalid(...)`) and exact terminal projection back to [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) workflow outcomes and completion-style payloads. The contract distinguishes what must be exact for semantic conformance from what may remain conservative on staged runtime-adoption surfaces such as TASK-405 through TASK-412 retained/runtime observation slices, while keeping `Par` branch-state details, concrete runtime layouts, and full retained-completion parity out of scope for this slice. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md), [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md), and [docs/ideas/IMPLEMENTABILITY-REPORT.md](docs/ideas/IMPLEMENTABILITY-REPORT.md) now treat that execution-record contract as the Phase 67 runtime-facing packaging anchor for later `ash-interp`, `Par`, completion-parity, and differential-conformance work.

- Completed TASK-431 as a docs/reference/spec/planning pass for the current big-step / small-step / conformance corpus. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now names the current canonical semantic and observable authorities explicitly, separates semantic theorem targets from [SPEC-026](docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) implementation-conformance obligations, and packages the first proof-facing meta-properties for future Lean/reference work: terminal projection from [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) terminal configurations to [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) outcomes, progress-or-blocked classification goals, deterministic-fragment determinism targets, helper-bounded nondeterminism obligations, and preservation targets for cumulative `Ω`, `π`, `T`, and `ε̂`. The refreshed boundary also makes explicit how Lean should treat canonical specs, source/handoff contracts, SPEC-026, and historical planning/evidence artifacts without promoting old phase notes into semantic authority. Planning/task surfaces were updated accordingly.

- Completed TASK-430 as a docs/spec/reference planning pass for [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). The small-step spec now freezes one explicit helper-contract package and one proof-usable state taxonomy across the small-step/runtime correspondence story: progress transitions, blocked/suspended waiting, terminal success, terminal rejection/failure, and invalid/inadmissible/runtime-failure boundaries are now distinguished directly; helper-owned contracts are packaged explicitly for receive-arm selection, parallel terminal aggregation, policy decision/rejection ownership, obligation transition/discharge and scoped reconciliation ownership, spawned-child completion sealing/observation ownership, and the remaining already-frozen v1 atomic helper boundaries. The update keeps compatibility with [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) and TASK-405's runtime classification surface without flattening helpers into Rust APIs, and aligns nearby planning/reference/reporting surfaces accordingly.

- Completed TASK-429 as a docs/spec-only proof-usability pass for [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). The small-step spec now presents explicit canonical workflow rule definitions rather than only rule-family inventory prose, including terminal/structural, binding/branching, capability-policy-obligation, modal/fallback, and receive/concurrency rule groups. The rewrite adds specification-only residual-form notation to make premises, propagation, and terminal shape directly citable while preserving the accepted v1 boundaries: expressions and patterns remain atomic, helper-owned receive/guard/obligation/provenance/parallel boundaries remain helper-owned, and `Par` stays interleaving-compatible with helper-backed aggregation instead of being collapsed into fake sequential machine rules. Nearby planning/reference surfaces now describe `SPEC-025` as the proof-usable rule-definition surface for later conformance and formalization work, while current runtime evidence remains honestly partial for cumulative carriers, retained completion packaging, and fully explicit helper-backed `Par` aggregation.

- Completed TASK-428 as a docs/spec-only conformance-contract pass. The repo now includes [SPEC-026](docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) as the explicit cross-implementation contract for Ash, freezing three canonical conformance surfaces: big-step / terminal semantic conformance, small-step / state-taxonomy conformance, and runtime-observable conformance. The new contract makes the authority hierarchy explicit, defines what each surface must preserve, bounds allowed nondeterminism for helper-owned concurrency and `receive`, explains how differential-testing artifacts must compare implementations when exact step ordering is not required, and keeps honest wording that current Rust runtime evidence remains partial for cumulative carriers, retained completion parity, uniform blocked/suspended packaging, and fully explicit helper-backed `Par` aggregation. Nearby reference/planning/ideas/spec-index surfaces now treat Phase 67 as having one explicit conformance anchor.

- Added Phase 67 planning for formal conformance and runtime carrier alignment. The new plan introduces TASK-428 through TASK-440 as a contract-first queue covering implementation conformance, proof-usable `SPEC-025` rule definitions, helper/state-taxonomy clarification, semantic execution-record contracts, runtime carrier follow-ons in `ash-interp`, canonical IR semantics corpus design, differential conformance harness work, and Lean/reference refresh planning.

- Added planned task files for TASK-433 through TASK-440 covering the `ash-interp` execution-record substrate, `Par` branch-state and runtime aggregation work, retained-completion parity contract/follow-on work, canonical IR semantics corpus and result-format definition, Rust-first differential conformance harness work, and Lean/reference refresh planning against the current semantic corpus.

- Added planned task files for TASK-428 through TASK-432 covering the implementation-conformance contract, full `SPEC-025` rule definitions, small-step helper contracts and state taxonomy, formalization-boundary refresh, and the semantic execution-record / terminal-projection contract.

- Completed TASK-427 as the faithful closeout and corpus-alignment pass for [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). The small-step spec now states directly that it is the docs/spec home for the accepted workflow-first small-step contract, keeps [MCE-005](docs/ideas/minimal-core/MCE-005-SMALL-STEP.md) and [MCE-006](docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md) as the design/evidence backplanes, and preserves honest wording that current runtime support remains partial for cumulative carriers, retained completion packaging, and fully explicit helper-backed `Par` aggregation; nearby plan/index/ideas/reporting surfaces were aligned accordingly.

- Added [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), a workflow-first small-step operational semantics spec that distills the accepted MCE-005 / TASK-395 / TASK-396 corpus into the docs/spec surface. It presents the small-step judgment, configuration contract, observability split, blocked-vs-stuck distinction, canonical workflow rule inventory, and terminal correspondence boundary back to [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) without superseding the accepted MCE-005 backbone.

- Completed TASK-426 as a docs/spec audit pass for [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). The Phase 66 audit now freezes an explicit `SPEC-025 -> SPEC-004` compatibility matrix, an explicit `SPEC-025 runtime-facing claims -> MCE-006` evidence matrix, and a final conservative verdict: `SPEC-025` is faithful and compatible, but current runtime evidence remains partial for cumulative carriers, retained completion packaging, and full helper-backed `Par` aggregation.

- Completed TASK-425 as a docs/spec consolidation pass for [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). The small-step spec now makes its normative vs informative split explicit, presents rule families as normative inventory/intent markers rather than full formal schemata, states helper names as schematic ownership markers instead of mandatory Rust APIs, and tightens helper-boundary wording to stay faithful to [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) and accepted [MCE-005](docs/ideas/minimal-core/MCE-005-SMALL-STEP.md). The accepted blocked/suspended vs stuck distinction, v1 atomic expression/pattern boundaries, and `Par` stance of interleaving progress plus helper-backed terminal aggregation without left-to-right collapse are preserved.

- Completed TASK-423 in `ash-typeck` as the workflow binding propagation follow-on for the closed-world interfaces MVP. Workflow-side validation and declared return inference now derive `For`-bound pattern types from the collection element type instead of manufacturing unrelated fresh variables, `Observe` bindings no longer leak fresh variables into later declared-return checking, non-list `For` collections now fail honestly with an explicit error instead of silently fabricating an element type, and surfaced `Propose.binding` is now rejected explicitly with an MVP-specific diagnostic rather than only failing indirectly later in checking. Focused regression coverage now exercises `For`-bound canonical interface calls, `For`-bound declared returns, non-list `For` rejection, honest `Observe` failure behavior, and explicit MVP rejection of surfaced `Propose.binding`.

- Completed TASK-422 in `ash-typeck` as the closed-world interfaces MVP semantic pass. The typechecker now registers top-level interface and impl declarations in dedicated environments, rejects duplicate impls for the same `(Interface, ConcreteNominalType)` pair, validates canonical workflow bounds of the form `T: Interface`, typechecks impl method bodies against interface signatures, enforces declared workflow return types at the workflow/program entrypoints, and resolves canonical method calls `Interface::method(value)` across both direct arguments and match-bound pattern variables. Coverage now includes coherence checks, bounded-generic canonical call resolution, impl-body signature validation, pattern-bound method-call typing, and declared return-type mismatch rejection through `type_check_workflow_def(...)` and `type_check_program(...)`.

- Completed TASK-421 in `ash-core` and `ash-parser` as a strict-TDD parser/AST substrate slice for the frozen closed-world interfaces MVP. The parser surface and core metadata carriers now represent explicit interface declarations, explicit impl declarations, constrained workflow generic parameters in canonical `T: Interface` form, and explicit namespaced method calls in canonical `Interface::method(value)` form. Parser coverage now includes accepted interface/impl/bound/call shapes plus rejection of obviously malformed interface and impl syntax, while lowering remains explicitly honest about the task boundary by rejecting interface method-call lowering until TASK-422 rather than silently fabricating semantics.

- Completed TASK-420 as a contract-first decision pass after the landed TASK-419 alignment. After inspecting the current promoted effect contract, repo surfaces, and implementation footprint, Ash explicitly defers adding a surfaced `Pure` bottom lattice element for now and keeps the current four-grade model (`Epistemic < Deliberative < Evaluative < Operational`). Control/modal forms therefore continue to be described as not adding a surfaced grade of their own rather than silently normalizing to a new fifth grade, and the planning/task bookkeeping now records that decision clearly without widening code or runtime contracts.

- Completed TASK-419 in `ash-typeck` as a strict-TDD alignment pass for the promoted coarse effect contract. Workflow-form inference now treats `For`, `Ret`, and `Oblige` as control/governance forms that do not introduce stronger surfaced grades on their own, preserving join-based composition over the existing four-grade lattice. Runtime effect verification now exposes a type-derived `check_inferred(...)` path and the aggregate verification flow uses that preclassified workflow effect instead of treating provider-side metadata as the source of truth. Requirement checking now records provider effect metadata as compatibility-only metadata, rejects weaker provider metadata when it undershoots source classification, and preserves source-level capability classification when provider metadata overreaches upward. Coverage now includes workflow-form classification, join-based composition, provider metadata compatibility rejection, source-level classification winning over provider metadata overreach, and runtime verification over preclassified effects.

- Completed TASK-418 across `ash-interp`, `ash-core`, and entry/runtime surfaces by closing the runtime loop for tuple variants and reconciling the remaining concrete `RuntimeError` drift. Tuple constructor expressions now evaluate into ordinary variant values that preserve canonical positional payload order; runtime tuple-variant patterns now match positionally and reject arity drift; observable value formatting now renders tuple variants as `Name(v0, v1, ...)` instead of leaking synthetic `_0`/`_1` field names; and the stdlib-visible `RuntimeError`/entry exit-code contract now consistently uses tuple-variant syntax (`RuntimeError(Int, String)` and `RuntimeError(code, _)`) across interpreter tests, stdlib files, parser/typechecker regression surfaces, engine exit-code derivation, and changelog/docs updates. Coverage now includes tuple constructor evaluation, tuple-pattern runtime matching, nested tuple-pattern extraction, runtime tuple display, exact tuple arity enforcement, and tuple-shaped `RuntimeError` contract checks.

- Completed TASK-417 in `ash-typeck` and lowering by finishing tuple-variant lowering/type metadata/typechecking/exhaustiveness support without regressing unit or record variants. Tuple enum-variant declarations, constructor expressions, and variant patterns now preserve canonical positional payload shape through lowering and type-environment metadata; tuple constructors are typechecked by positional arity and payload type; tuple variant patterns bind payload positions by order; and non-exhaustive witness reporting now preserves tuple witness formatting such as `RuntimeError(_, _)` instead of collapsing tuple variants to bare constructor names. Coverage now includes tuple constructor success, tuple arity/type mismatch rejection, tuple-pattern binding, expected-ADT pattern typing, and tuple witness shape preservation, alongside the required helper/test-fixture migrations to the new payload-bearing AST and ADT metadata.

- Completed TASK-416 in `ash-parser` by teaching the parser and surface/source AST substrates to preserve tuple enum-variant shape distinctly from existing unit and record variants. Type definitions now parse tuple payload declarations such as `RuntimeError(Int, String)`, constructor expressions now preserve tuple payloads such as `RuntimeError(2, "missing config")` without collapsing them into record constructors, and variant patterns now preserve tuple destructuring such as `RuntimeError(code, msg)` including nested tuple-pattern structure. Parser regression coverage now includes tuple-variant declarations, tuple constructor expressions, tuple variant patterns, nested tuple variant patterns, and rejection of malformed named-field syntax inside tuple payload forms.

- Added a concrete post-promotion implementation queue for the type-system work that followed TASK-413 / TASK-414 / TASK-415. Phase 65 in `docs/plan/PLAN-INDEX.md` now sequences tuple-variant parser/AST work (TASK-416), tuple-variant lowering/typechecking/exhaustiveness (TASK-417), tuple-variant runtime support plus `RuntimeError` reconciliation (TASK-418), effect inference/runtime-verification alignment (TASK-419), optional `Pure` bottom-effect follow-on (TASK-420), and the first two closed-world interfaces MVP implementation slices for parser/AST substrate and typechecker coherence/method resolution (TASK-421, TASK-422).

- Completed TASK-415 as a docs/spec-only narrowing pass for ad-hoc polymorphism. The corpus now makes the `TYPES-002` relationship explicit: `v1` remains the preserved reasoning trace, `TYPES-002 V2` remains the broader polished exploration, and `docs/ideas/type-system/TYPES-002-v2-mvp-cut.md` is the narrowed follow-on target for planning/spec work. The MVP cut now fixes one canonical bound form (`T: Interface`), one canonical method-call form (`Interface::method(value)`), a strict non-overlapping impl coherence rule, explicit capability/interface separation, and an effect-conservative first pass that defers open-world typeclasses, associated types, associated effects, dynamic dispatch / trait objects / existential packaging, and capability/interface unification. `docs/ideas/README.md`, `docs/ideas/IMPLEMENTABILITY-REPORT.md`, `docs/plan/PLAN-INDEX.md`, and the TASK-415 record now reflect that narrowed target and mark Phase 64 complete.

- Completed TASK-414 as a docs/spec-only convergence pass for the promoted type-system packet. The corpus now records one narrow coarse effect-typing contract: workflow effect classification is computed from canonical workflow forms and source-level contracts; provider effect metadata is compatibility/validation metadata rather than the primary source of source-level effect typing; composition remains join-based over the current coarse lattice; and the `Pure` bottom-element question is recorded as explicit follow-up instead of silently treated as already normative. The update also tightens promoted vocabulary usage across the main affected docs (`capability declaration`, `capability identity`, `capability witness`, `provider`, `effect classification`, `policy context`, `obligation context`, `provenance context`), adds workflow-form classification tables to the reference/type-system corpus, marks `TYPES-003` and `TYPES-004` as promoted candidate reasoning records, and closes TASK-414 in `PLAN-INDEX.md`.

- Added a contract-first type-system promotion packet around the `docs/ideas/type-system/` explorations: `TYPES-001` now selects explicit parenthesized tuple-variant syntax as the canonical source form and links to new [TASK-413]; the repo now includes `docs/reference/type-system-vocabulary-guidance.md` as reusable cleanup guidance promoted from `TYPES-003`; `docs/ideas/type-system/TYPES-002-v2-mvp-cut.md` narrows `TYPES-002 V2` into a coherence-first closed-world interfaces MVP cut; and new planning tasks [TASK-413], [TASK-414], and [TASK-415] plus Phase 64 in `docs/plan/PLAN-INDEX.md` capture the next docs/spec promotion work for tuple variants, effect/vocabulary cleanup, and closed-world interfaces MVP scoping.

- Completed TASK-412 in `ash-interp` by adding one dedicated retained-completion wait API alongside the existing lookup surface: `RuntimeState::wait_for_retained_completion(&ControlLink) -> Result<RetainedCompletionRecord, ControlLinkError>`. The new wait path reuses the existing sealed retained completion carrier rather than inventing a parallel payload type, returns immediately for already-sealed records, resolves for both child-owned completions and control tombstones, and preserves first-write authority by waiting on the same write-once retained record sealed through `ControlLinkRegistry`. Invalid or unregistered targets remain distinguishable as `ControlLinkError::NotFound(...)` instead of synthesizing fake completion payloads. Tests now cover child-completion waits, kill/tombstone waits, already-sealed immediate reads, and non-hanging invalid-target behavior. This implementation remains intentionally narrow and additive: it improves retained-completion observation ergonomics without claiming full `CompletionPayload` parity or broader cumulative carrier closure.

- Completed TASK-411 in `ash-interp` by enriching the sealed retained completion carrier with one conservative `CompletionPayload.provenance`-like slice: `RetainedCompletionRecord.provenance: Option<ConservativeRetainedProvenanceSummary>` plus `RetainedCompletionRecord::conservative_provenance_summary()`. Child-owned retained completions now preserve the narrowest honest runtime-owned provenance snapshot available today: child `workflow_id`, optional immediate `parent_workflow_id`, and retained spawn `lineage()` drawn from runtime-owned spawn registration rather than claimed full terminal `π'` transport. Control tombstones remain distinct as `RetainedCompletionKind::ControlTerminated` with `result: None`, `effects: None`, `obligations: None`, and `provenance: None`, and first-write sealing remains authoritative. This implementation deliberately does not claim exact full `CompletionPayload.provenance` parity or broader cumulative provenance/trace closure; it only retains the runtime-owned identity/lineage slice the current spawned-child lifecycle can actually snapshot.

- Completed TASK-410 in `ash-interp` by enriching the sealed retained completion carrier with one honest `CompletionPayload.obligations`-like slice: `RetainedCompletionRecord.obligations: Option<ConservativeRetainedObligationsSummary>` plus `RetainedCompletionRecord::conservative_obligations_summary()`. Child-owned retained completions now preserve the narrowest terminal-visible obligation state the runtime can honestly snapshot today: local pending obligations visible in the observed terminal child context plus active-role pending/discharged obligations visible through `RoleContext`, while control tombstones remain distinct as `RetainedCompletionKind::ControlTerminated` with `result: None`, `effects: None`, and `obligations: None`. This implementation deliberately does not claim exact full `CompletionPayload.obligations` parity or broader cumulative `Ω` closure: the retained obligations slice reflects only the terminal observation path the current runtime can actually snapshot. Tests now cover retained obligations summaries for successful and failing spawned-child completions, write-once stability with obligations present, and continued tombstone distinction. Docs/reporting surfaces now record that obligations retention has landed while provenance, exact effect transport, exact full obligations parity, dedicated completion-wait semantics, and broader cumulative carrier packaging remain open.

- Completed TASK-409 in `ash-interp` by enriching the sealed retained completion carrier with one conservative `CompletionPayload.effects`-like slice: `RetainedCompletionRecord.effects: Option<ConservativeRetainedEffectSummary>` plus `RetainedCompletionRecord::conservative_effect_summary()`, where `ConservativeRetainedEffectSummary` currently exposes `terminal()` and `reached()`. Child-owned retained completions now preserve a retained effect summary with `effects.terminal_upper_bound` and conservative `effects.reached_upper_bound`, while control tombstones remain distinct as `RetainedCompletionKind::ControlTerminated` with both `result: None` and `effects: None`. This implementation deliberately does not transport the full trace `T` or claim exact `CompletionPayload.effects` parity: the retained reached-effect set is a conservative workflow-form-derived summary, and the retained terminal effect is a conservative runtime-derived upper-bound summary. Tests now cover retained effect summaries for successful and failing spawned-child completions, conservative multi-effect retention, write-once stability with effect summaries present, and continued tombstone distinction. Docs/reporting surfaces now record that effect-summary retention has landed while obligations, provenance, exact effect transport, dedicated completion-wait semantics, and broader cumulative carrier packaging remain open.

- Completed TASK-408 in `ash-interp` by enriching the sealed retained completion carrier with one honest `CompletionPayload.result`-like field: `RetainedCompletionRecord.result: Option<Box<ExecResult<Value>>>` plus `RetainedCompletionRecord::terminal_result()`. Child-owned retained completions now preserve the direct terminal success value or terminal `ExecError`, while control tombstones remain distinct as `RetainedCompletionKind::ControlTerminated` with `result: None`; the coarse `RuntimeOutcomeState` surface remains in place alongside this richer payload slice, and write-once sealing is preserved. Tests now cover direct retained success payloads, direct retained failure payloads, write-once stability with richer payload contents, and explicit distinction between control tombstones and child-owned payloads. Docs/reporting surfaces now record that richer retained result data has landed while obligations, provenance, effects, and broader cumulative carrier packaging remain open.

- Completed TASK-407 in `ash-interp` by tightening the real spawned-child execution substrate keyed by `workflow_type`: `kill` and child-side completion sealing now compete through one authoritative terminal transition path in `ControlLinkRegistry`, so the true first terminal event wins; `Workflow::Spawn` now returns live control authority only when a runtime-owned child workflow is actually registered, instead of producing a live-looking orphan control target; and automatic child-side completion sealing now keeps benign completion-vs-kill races quiet while surfacing unexpected seal failures instead of swallowing them broadly. The evaluated spawn `init` value still passes through the conservative child entry contract by binding it as `init` in child context, and the runtime still avoids any claim of full `SPEC-004` `CompletionPayload` parity or broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure. Tests now cover honest unregistered-spawn behavior, real child execution, automatic retained completion sealing for both success and failure, stable write-once sealing after automatic capture, and the fixed completion-before-kill terminal ordering.

- Followed up TASK-406 in `ash-interp` after review by making retained completion records sealed/write-once in `ControlLinkRegistry`, preserving the first terminal tombstone on `kill`, and removing the eager inline spawned-child termination/regression from `Workflow::Spawn` so returned control links remain live and useful for pause/resume/check-health/kill. That TASK-406 slice kept the retained carrier at `RetainedCompletionKind::{Completed, ControlTerminated}` and surfaced it through `RuntimeState::{register_spawned_control_link, record_control_completion, retained_completion}` without yet wiring automatic capture from a real spawned-child lifecycle; TASK-407 later adds that missing runtime-owned child execution path. This continues to avoid claiming full `SPEC-004` `CompletionPayload` parity or broader cumulative `Ω` / `π` / `T` / `ε̂` packaging closure.

- Completed TASK-405 in `ash-interp` by introducing the public `RuntimeOutcomeState` classification with the conservative classes `TerminalSuccess`, `Active`, `BlockedOrSuspended`, `InvalidOrTerminated`, and `ExecutionFailure`; wiring `ExecError`, `ControlLinkError`, `LinkState`, and `RuntimeState` control-link visibility into that authoritative runtime surface; adding focused tests for suspended, invalid/terminated, execution-failure, terminal-success, and runtime-state control-link cases; and updating the MCE-007 / MCE-008 planning-reporting corpus to record this as the first runtime-side follow-on for the frozen blocked/terminal/invalid residual drift item without claiming closure of cumulative carriers, retained completion payloads, or helper-backed `Par` aggregation.

- Reconciled TASK-397 as completed framing/scaffold work for MCE-007 by marking the task and Phase 62 planning surfaces complete, recording that its intended outputs were materially realized by the published MCE-007 matrix / residual-gap / closeout corpus, and preserving the conservative note that true runtime-side residual drift remains open.

- Completed TASK-400 as documentation/planning/full-stack closeout work for MCE-007, adding a final closeout/signoff/drift-prevention section to `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`, freezing the accepted five-layer matrix state and current residual register, explicitly preserving the mixed sequencing / binding / branching row as accepted local execution alignment plus unresolved cumulative-carrier drift, publishing signoff conditions that distinguish closeout completion from full runtime closure, and updating the surrounding planning/reporting corpus to reflect that the closeout artifact is complete while true residual runtime drift remains open.

- Completed TASK-399 as documentation/planning/full-stack alignment work for MCE-007, adding a dedicated residual-gap classification layer to `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`, freezing the categories `packaging-only`, `accepted partiality`, and `true residual drift`, assigning owners to every remaining non-closed issue, and distinguishing accepted owner-bound limitations from the true residual drift set around blocked-state classification, cumulative semantic-carrier packaging, retained completion observation, and helper-backed `Par` aggregation.

- Completed TASK-398 as documentation/planning/full-stack alignment work for MCE-007, ingesting the frozen MCE-006 Phase 63 runtime-evidence packet into `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`, replacing the old Small-step → Interpreter placeholders with row-level conservative classifications, and updating the surrounding planning/reporting corpus to reflect that runtime-evidence ingestion is now complete while cumulative carriers, blocked-state unification, retained completion payloads, and full helper-backed `Par` aggregation remain explicit follow-up gaps.

- Completed Phase 63 / TASK-404 as documentation/planning/runtime-correspondence closeout work for MCE-006, adding a dedicated observable-preservation / divergence-taxonomy / MCE-007-handoff section to `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`, freezing a conservative checklist for return/non-success status, blocked-vs-terminal-vs-invalid boundaries, `Ω`, `π`, `T`, and `ε̂`, and concluding that the current interpreter only partially realizes the accepted MCE-005 backbone for observable purposes because authoritative cumulative carriers and retained completion-style payloads remain partial or missing.

- Completed Phase 63 / TASK-403 as documentation/planning/runtime-correspondence work for MCE-006, adding a dedicated `Par` correspondence section to `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md` that records the current `Workflow::Par` operational model as bulk async child execution via `join_all(...)`, identifies cloned `Context` state as the main branch-local carrier while mailbox/control/proxy/suspension infrastructure remains shared, and concludes conservatively that successful terminal child values are directly aggregated into `Value::List(...)` but full helper-backed cumulative-state aggregation for `Ω`, `π`, `T`, and `ε̂` is still only partial/missing rather than fully realized.

- Completed Phase 63 / TASK-402 as documentation/planning/runtime-correspondence work for MCE-006, adding a dedicated operational correspondence section to `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md` for residual control, blocked/suspended state realization, and completion/control authority, explicitly classifying active vs blocked vs terminal vs invalid runtime-facing states, recording direct vs distributed vs weak/missing realization boundaries, and conservatively concluding that `ControlLinkRegistry` directly realizes reusable/terminal control lifecycle while retained `SPEC-004` completion payload support remains only partial/indirect on the inspected runtime path.

- Completed Phase 63 / TASK-401 as documentation/planning/runtime-correspondence work for MCE-006, adding a canonical semantic-carrier → runtime mapping table to `docs/ideas/minimal-core/MCE-006-SMALL-STEP-IR.md`, classifying the current interpreter as a hybrid control representation, and recording first-pass safe indirections, documentation gaps, and correspondence risks for `A = (C, P)`, `Γ`, `Ω`, `π`, `T`, `ε̂`, residual workflow/control state, and terminal result classes.

- Completed Phase 61 / TASK-394 / TASK-395 / TASK-396 as a documentation/planning closeout for MCE-005, creating the missing Phase 61 task records, promoting `docs/ideas/minimal-core/MCE-005-SMALL-STEP.md` from an exploratory note to an accepted small-step semantic backbone over canonical `SPEC-001` workflow configurations, fixing the chosen workflow-step judgment and configuration/label observability split, recording blocked-vs-stuck behavior plus the canonical workflow rule inventory, and updating `MCE-006`, `MCE-007`, the ideas index/reporting corpus, and the plan corpus so MCE-006 is no longer framed as blocked on undefined small-step foundations.

- Completed TASK-393 / MCE-004 as a documentation/planning closeout, adding `docs/plan/tasks/TASK-393-big-step-semantics-alignment.md`, promoting `docs/ideas/minimal-core/MCE-004-BIG-STEP-ALIGNMENT.md` to accepted status, and recording the resolved surface → canonical IR → big-step alignment decisions: `Workflow::Seq` stays primitive, `Par` aggregates successful branch effects by join with helper-backed concurrent aggregation, spawned children seal their own authoritative terminal state in `CompletionPayload`, and `match` remains primitive while `if let` lowers to `Expr::Match` with a wildcard fallback arm.
- Completed TASK-370 / MCE-002 with a formal IR audit report at `docs/ideas/minimal-core/MCE-002-IR-AUDIT-REPORT.md`, identifying `crates/ash-core/src/ast.rs` as the de facto primary core-AST carrier and recommended future source of truth for the core layer, documenting the current 30 Workflow and 13 Expr forms plus related helper carriers, rejecting `Workflow::Seq` elimination, confirming `Expr::IfLet` as sugar over `Match`, identifying duplication across `workflow_contract.rs`, `stream.rs`, and the active parser-surface/typechecker representation path as the highest-value consolidation target, and proposing a conservative minimal-core direction that defers deeper form eliminations until semantics/lowering are cleaner.

### Fixed

- Fixed the `ash-interp` property test `prop_discharged_set_contains_all_discharged` to generate only truly undeclared extra obligations instead of allowing collisions with role-declared obligations that produced invalid counterexamples during TASK-433 verification.

- Fixed TASK-412 planning/reporting consistency across `docs/plan/PLAN-INDEX.md`, `docs/ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md`, and `docs/ideas/minimal-core/MCE-008-RUNTIME-CLEANUP.md` so those corpus surfaces now reflect the landed retained-completion wait API instead of still describing dedicated completion waiting as open.

- Fixed the `ash-interp` property test `prop_obligation_discharge_order_independent` to generate unique obligation sets via the existing helper instead of duplicate names that falsely turned an order-independence property into a double-discharge failure (`AlreadyDischarged`).

- Added deterministic per-task worktree metadata and fail-closed provisioning to `tools/agent-pipeline`, including persisted manifest assignments, repo-root `.worktrees/<TASK-ID>` derivation, supervisor launch gating, and worktree-based stage execution without moving task-bundle artifacts (TASK-378).

- Added a native `ash-pipeline retry-feedback` helper to `tools/agent-pipeline` so blocked tasks with `feedback-resolution.md` can be explicitly released back to `queue` or `in-progress`, with inferred restart stages from review artifacts (including archived `retry-history/.../*.review` paths on later retry cycles), preservation of the newest live review when repeated retries occur, archived review provenance under `retry-history/`, stale downstream artifact/log cleanup, dependency-safe direct restore checks, task-bundle path validation for referenced review artifacts, matching prompt-time validation when guidance files are consumed, and matching Vila wrapper/README support (TASK-386).

- Restored portable packaged service configuration for `tools/agent-pipeline` by removing the checked-in host-specific NVM PATH entry from `agent-pipeline.service`, bringing packaging tests back to green while preserving explicit workspace/state environment variables.

- Hardened CLI task-id validation so non-queue filesystem-touching commands (`status --task`, `pause`, `resume`, `abort`, `steer`, `resolve-feedback`, `retry-feedback`, `logs`, and `events`) now reject path-style task ids instead of only validating them during queueing, and tightened supervisor start gating so already in-progress tasks with unmet dependencies are not launched during restart/recovery sweeps.

- Hardened task dependency handling so `ash-pipeline queue --depends-on ...` trims duplicate/whitespace dependency ids, rejects self-dependencies, rejects task ids or dependency ids containing path-separator traversal syntax, and fails fast on dependency cycles instead of silently creating permanently stuck queue entries (TASK-383).

- Added structured feedback-resolution support to `tools/agent-pipeline` so operators can persist `feedback-resolution.md` via `ash-pipeline resolve-feedback`, require that it references an existing supported retry review artifact already present in the task bundle, write that artifact file explicitly as UTF-8 for consistency with later readers, refresh `updated_at` when `retry-feedback` mutates manifest state, surface feedback-resolution metadata in status output, include both the resolution and original review artifact in retry prompt context, and expose the same flow in the Vila wrapper and README without automatic queueing (TASK-385).

- Completed Phase 59 agent-pipeline worktree isolation: stages now execute against task worktrees with explicit dual-root prompt contracts, status/dashboard surfaces expose persisted worktree path/branch metadata, `cleanup-worktree` safely removes blocked/done task worktrees, now re-validates deterministic task worktree assignment/containment before any removal, reports invalid persisted worktree metadata distinctly from absent metadata, clears manifest metadata after successful removal, supervisor/worktree provisioning rejects unsafe persisted task ids, stale git-worktree reuse entries with missing directories are pruned before deterministic reprovision or blocked if prune fails, cleanup derives repo roots robustly from persisted worktree metadata when only `--base-dir` is supplied, base-dir-only cleanup now rejects malformed absolute worktree paths cleanly, prune failure after successful removal no longer leaves stale manifest worktree metadata behind, missing configured workspace roots now fail closed instead of crashing supervisor flows, supervisor now honors configured workspace roots for provisioning instead of heuristic repo rediscovery, aggregate text status surfaces malformed worktree metadata, and closeout tracking/docs now mark TASK-378 through TASK-382 complete.

- Switched the default `tools/agent-pipeline` stage-agent mapping to Hermes for every stage so normal pipeline execution no longer depends on Codex tokens, added native Hermes CLI launch commands for the previously Codex-default stages, preserved explicit `--stage-agents` / `AGENT_PIPELINE_STAGE_AGENTS` overrides for optional Codex reassignment, and updated pipeline/Vila docs to reflect the Hermes-first runtime contract (TASK-387).

- Tightened the Vila wrapper queue flow so missing or ambiguous `docs/plan/tasks/TASK-XXX-*.md` auto-discovery now fails closed instead of silently queueing a task without `--from-spec`, and updated the Vila integration guide to document the stricter queue semantics plus the newer `resolve-feedback`, `retry-feedback`, and `logs` operator flows.

- Added live per-stage stdout/stderr persistence to `tools/agent-pipeline`, exposed `ash-pipeline logs` plus matching Vila wrapper support for peeking at active stage output, added true `--follow` tailing for newly appended log chunks, and documented deterministic `<stage>.stdout.log` / `<stage>.stderr.log` task-bundle log files while preserving existing post-exit result handling (TASK-384).

- Added task dependency gating to `tools/agent-pipeline` so queued task manifests can persist prerequisite task ids, `ash-pipeline queue --depends-on ...` can declare them explicitly, queued tasks remain blocked in queue until every dependency is done/complete, and status output now surfaces unmet dependencies clearly without changing normal behavior for independent tasks (TASK-383).

- Updated repository ignore rules so local `.agents` runtime state, Python cache directories, Ruff/Pytest caches, `__pycache__`, `*.py[cod]`, `*.egg-info`, and `tools/agent-pipeline/REPLACE_TMPDIR` no longer appear as untracked noise during agent-pipeline development (TASK-377).

- Updated `tools/agent-pipeline` so the supervisor persists its effective stage-agent mapping into `status/dashboard.json`, `ash-pipeline status --format json` prefers that runtime mapping when available, and invalid `--stage-agents` or `AGENT_PIPELINE_STAGE_AGENTS` input now fails with concise Click-facing errors instead of uncaught tracebacks (TASK-376).

- Exposed the effective `tools/agent-pipeline` stage-agent mapping in `ash-pipeline status --format json`, so runtime agent overrides are directly observable from the status surface without changing text-mode behavior (TASK-375).

- Made `tools/agent-pipeline` stage-agent selection configurable at runtime via shared CLI/supervisor/spawner validation, preserving the default stage graph plus existing prompt and artifact contracts while rejecting invalid stage or agent overrides clearly (TASK-374).

- Upgraded `tools/agent-pipeline` to use shared prompt-contract fragments, stricter design/spec/plan/impl/qa/validate artifact expectations, and fail-closed QA/validate review blocking without changing the external stage graph (TASK-373).

- Fixed the packaged `tools/agent-pipeline` deployment so installer and Vila helper scripts derive clone-local paths, the systemd unit sets explicit workspace/state environment variables with sandbox writes that match `impl` needs, and `queue --from-spec` now rejects missing input before creating task state (TASK-372).

- Hardened `tools/agent-pipeline` supervision so staged agents now launch asynchronously, task bundles move as full directories with colocated context files, status lookups include completed tasks, abort/steer controls persist correctly, and agent execution no longer depends on a hard-coded Ash workspace path (TASK-371).

- TASK-370/MCE-002 documentation: marked `Seq` elimination as **rejected**; fixed
  `Workflow::Split` description; converted all absolute paths to repo-relative; reframed
  Task 4 to remove incorrect `Orient` binding language; aligned MCE-002 Seq status with
  TASK-370 conclusion.

- Added the initial `runtime` stdlib surface under `std/src/`, including `RuntimeError`,
  the `Args` capability declaration, and a minimal supervisor scaffold for entry-point work
  (TASK-359).

- Defined the canonical `runtime::RuntimeError` stdlib type as a single-variant ADT with
  `exit_code` and `message` fields for `Result<(), RuntimeError>` entry-point contracts
  (TASK-360).

- Defined SPEC-004 control-link completion payload semantics (TASK-S57-1),
  including runtime-internal supervisor observation, `CompletionPayload`/`EffectTrace`, and
  terminal-control outcomes for spawned workflow completion.

- Defined SPEC-005 `ash run` exit-immediately policy (TASK-S57-2), including
  `ash run <file> [-- <args>...]`, `main`-derived exit codes, and the explicit
  boundary that descendant workflows do not extend process lifetime.

- Defined SPEC-021 observable exit behavior (TASK-S57-3), tying external
  process exit to `main` completion, clarifying that descendant fate after exit
  is non-observable and implementation-defined, and aligning the observable
  boundary with SPEC-004 and SPEC-005.

- Added minimum `ash-cli` entry integration coverage for canonical success,
  declared runtime-error exit-code propagation, missing-`main` diagnostics, and
  injected runtime `Args` handling, closing the required Phase 57 minimum test
  slice (TASK-368a).

### Changed

- Removed `Par` from the active Ash language contract. The canonical sequential workflow contract now specifies that a single workflow in Ash is sequential, with concurrency and parallelism modeled at the system level through multiple communicating workflows. All normative `Par` contract references in [SPEC-001](docs/spec/SPEC-001-IR.md), [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-022](docs/spec/SPEC-022-WORKFLOW-TYPING.md), [SPEC-025](docs/spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md), and [SPEC-026](docs/spec/SPEC-026-IMPLEMENTATION-CONFORMANCE.md) have been amended to mark historical sections with \"(Historical)\" markers and remove normative language that would imply `Par` is part of the current active contract.

- Completed Task 4 by replacing all par-based examples, tutorials, and workflow fixtures with sequential composition or message-passing patterns. Removed `par` blocks from [examples/simple_workflow.ash](examples/simple_workflow.ash), [examples/multi_agent_research.ash](examples/multi_agent_research.ash), [examples/code_review.ash](examples/code_review.ash), [examples/04-real-world/customer-support.ash](examples/04-real-world/customer-support.ash), [tests/workflows/code_review.ash](tests/workflows/code_review.ash), and [tests/workflows/multi_agent_research.ash](tests/workflows/multi_agent_research.ash). Deleted [examples/02-control-flow/03-parallel.ash](examples/02-control-flow/03-parallel.ash) and updated [examples/02-control-flow/03-sequential.ash](examples/02-control-flow/03-sequential.ash) and [examples/02-control-flow/04-sequential.ash](examples/02-control-flow/04-sequential.ash) to demonstrate sequential composition without the removed `seq` keyword. Updated documentation in [docs/TUTORIAL.md](docs/TUTORIAL.md) (replaced \"Parallel Execution\" section with \"Sequential Composition\"), [docs/spec/SPEC-023-PROXY-WORKFLOWS.md](docs/spec/SPEC-023-PROXY-WORKFLOWS.md) (replaced par-based quorum example with sequential yield/resume), [examples/README.md](examples/README.md), [examples/02-control-flow/README.md](examples/02-control-flow/README.md), and [examples/workflows/40_tdd_README.md](examples/workflows/40_tdd_README.md) to remove all references to parallel execution and the `par` keyword.

- Corrected the README Phase 57 quick-start commands so the documented `ash run`
  and `ash run --trace` examples now point to real canonical entry files, while
  the larger `support_ticket` and `multi_agent_research` samples are labeled as
  reference-oriented workflows that need adaptation before they can run through
  the Phase 57 `main(...) -> Result<(), RuntimeError>` entry path.

- Redefined the default `ash run` path around the canonical Phase 57 entry bootstrap so normal execution, `--trace`, and dry-run now validate `main() -> Result<(), RuntimeError>`, accept trailing runtime args after `--` via injected `Args:<index>` providers, and keep `--output` producing an empty artifact for successful entry runs without a printable terminal value (TASK-365, TASK-366).

- Added a narrow engine-owned runtime stdlib registry keyed by canonical module path so `bootstrap_entry_source()` loads runtime stdlib through the engine and `parse_entry_source()` validates leading runtime imports before stripping the entry prelude; this remains limited to the entry/runtime stdlib slice rather than a general module graph (TASK-363a).

- Added narrow `ash-engine` entry bootstrap helpers that parse, check, verify, execute, and derive process exit codes from canonical `Result<(), RuntimeError>` entry workflow results, and wired `ash run` through that prerequisite slice only for obvious runtime entry sources while preserving ordinary workflow execution behavior; full TASK-366 CLI semantics remain downstream work (TASK-363b, TASK-363c, TASK-364).

- Added canonical entry workflow signature verification in `ash-engine` as a pure check over cached parsed workflow metadata, rejecting missing `main`, wrong return types, and non-capability parameters without starting bootstrap work (TASK-364).

- Completed the canonical `runtime::system_supervisor(args: cap Args) -> Int` stdlib contract, keeping spawn/completion observation runtime-internal for downstream bootstrap work while adding focused parser regressions for the exposed supervisor surface and workflow-body parse (TASK-362).

- Parser support now accepts canonical runtime capability parameters as `cap Args`, normalizes `observe Args 0` into the existing internal `Args:0` observe name used by capability checking, and adds focused parser plus parse-to-typecheck regression coverage for that entry-workflow surface (TASK-361).

- Aligned the downstream entry-point task docs with the `RuntimeError` single-variant ADT shape and added direct typechecker coverage for constructor composition plus interpreter coverage for nested variant-pattern extraction (TASK-360).

- Expanded the ad-hoc polymorphism exploration docs with a preserved `TYPES-002` review note and
  a new `TYPES-002 V2` synthesis document that cleans up dead ends, adds Ash-native examples,
  introduces decision-driving workloads, and clarifies that effects are a distinct typing
  dimension rather than ordinary value-level payloads.

- Clarified the `TYPES-002` and `TYPES-002 V2` exploration notes so authority elevation is framed
  explicitly as the gap between design authority and implementation authority, with v1 preserving
  three design choices and v2 recommending explicit source-level elevation sites backed by audit
  and provenance semantics.

- Added `TYPES-003`, a judgment-oriented exploration note that disambiguates capability
  declarations, capability witnesses, providers, effects, policies, obligations, and provenance
  so future Ash design discussions can use sharper language.

- Added `TYPES-004`, an effect-typing exploration note that treats the current lattice as Ash's
  coarse effect grade system, enumerates effect-producing workflow forms, frames provider
  metadata as compatible with but distinct from source-level effect typing, and proposes `Pure`
  as a surfaced bottom element for effect-neutral forms and normalized composition.

- Added the `OTP-001` and `OTP-002` exploration notes to git so the OTP case-study material is
  preserved alongside the type-system explorations and can inform later design work and examples.

- Clarified SPEC-009 and SPEC-012 so that Ash standard-library modules resolve
  from a compiler-provided root namespace and are imported with `::` syntax
  only; legacy dot-style import examples are invalid (TASK-S57-4).

- Clarified SPEC-017 so runtime-provided capability parameters use `cap <Identifier>` at
  usage sites while capability declarations remain `capability ...`; runtime injection occurs
  at workflow boundaries and read-like capability use remains effect-first (`observe Args 0`)
  (TASK-S57-5).

- Clarified SPEC-022 and SPEC-003 so the designated program entry workflow is typed by a
  canonical `main` contract: exact return type `Result<(), RuntimeError>`, zero or more
  usage-site capability parameters `cap X`, and ordinary body-inferred effects (TASK-S57-6).

- Closed out Phase 57 task tracking and user-facing entry documentation by
  marking the minimum integration slice complete, updating README guidance for
  canonical `ash run` entry workflows, and recording verification-driven
  completion of the implementation phase while leaving TASK-368b deferred
  (TASK-369).

### Fixed

- Aligned `ash run` entry failure reporting with the Phase 57 contract so missing files, missing `main`, wrong entry return types, and non-capability entry parameters now surface direct user-facing diagnostics on stderr with exit code `1` instead of falling back to legacy workflow execution or generic CLI error reclassification (TASK-367).

- Preserve canonical entry detection for import-free `requires:`/`ensures:` clauses whose expressions reference identifiers like `capabilities`, so `ash run` keeps bootstrap exit semantics on valid entry workflows.

- Normalized narrow runtime entry import matching in `ash-engine` so supported bootstrap imports still validate when inline block comments or extra whitespace appear inside canonical paths like `result::Result` and `runtime::RuntimeError`, keeping the scope limited to the entry prelude rather than widening into general import parsing (TASK-363a).

- Tightened `ash run` entry-candidate detection so CLI bootstrap now keys off a structural
  leading `runtime`/`result` prelude or the first canonical `workflow main() -> Result<(), RuntimeError>`
  header, avoiding false positives from comments or string literals that merely mention
  `RuntimeError` while preserving verification routing for genuine entry files; the structural
  fallback now also tolerates canonical post-return header clauses such as `capabilities: []`
  before the workflow body so import-free entry workflows still take the bootstrap path (TASK-363c).

- Reviewed and aligned the downstream Phase 57B task plans with the completed S57-1 through
  S57-6 specs, correcting stale capability syntax, entry-signature assumptions, and stdlib path
  references before implementation begins (TASK-S57-7).

|- **Phase 57: Entry Point and Program Execution Planning**

- Established 7 SPEC-first tasks (S57-1 through S57-7) for entry point semantics
  - S57-1: SPEC-004 control-link completion payload semantics
  - S57-2: SPEC-005 CLI exit-immediately policy
  - S57-3: SPEC-021 observable exit behavior
  - S57-4: SPEC-009/012 stdlib import/namespace rules
  - S57-5: SPEC-017 runtime-provided capability syntax
  - S57-6: SPEC-003/022 entry workflow typing contract
  - S57-7: Post-SPEC-update review of implementation tasks
- Established 13 implementation tasks (359-369) with validation gates
  - Stdlib foundation: TASK-359, 360, 361, 362
  - Runtime bootstrap: TASK-363a, 363b, 363c, 364, 365
  - CLI integration: TASK-366, 367
  - Testing: TASK-368a (minimum), 368b (deferred), 369
- All tasks reference normative SPEC (not MCE) per project policy

|- Extended the normative `SPEC-004` runtime value domain and display contract to include `Float(f)` alongside `Int(i)`, keeping the proof-grade semantics aligned with the neighboring float-capable specs.

|- Added an exploratory workflow declaration/runtime behavior design note that centers workflow as a callable, workflow-backed capability with boundary contracts, and records obligation-boundary alternatives for future small-step semantics work.

- Added a proof-grade design, task, and implementation plan for revising `SPEC-004` into a complete big-step core semantics suitable for Lean-oriented proofs and later small-step refinement work.

- Normalized the `SPEC-004` semantic backbone with explicit front-matter algebra, runtime failure categories, and separate workflow, expression, pattern, and helper judgment contracts (TASK-350).

- Completed the canonical `SPEC-004` pure-expression section for the core `Expr` forms by adding explicit `IndexAccess`, `Unary`, `Binary`, and `Call` judgment rules plus helper-boundary ownership text (TASK-350).

- Completed the canonical `SPEC-004` pattern semantics in one `PAT-*` section, explicitly covering wildcard, variable, literal, tuple, list, record, variant, duplicate-binder, and non-match versus rejection behavior while demoting legacy `bind(...)` prose to a historical note (TASK-350).

- Tightened `SPEC-004` pattern integration by routing `match`, `receive`, `observe`, and `let` through the canonical `⊢p` judgment and helper contracts, with explicit `PatternBindFailure` ownership for required-binding sites (TASK-350).

- Added normative propagation, lookup-failure, and post-lowering conventions to `SPEC-004` so rejection ownership, trace/effect preservation, and malformed-runtime handling have one proof-facing home (TASK-350).

- Extracted a dedicated `SPEC-004` helper-contract summary covering lookup, receive selection, action performance, obligation checking, parallel outcome combination, and provenance/trace helper laws (TASK-350).

- Clarified `SPEC-004` with explicit determinism/nondeterminism, semantic invariants, and proof-target/conformance sections, and aligned the formalization boundary note with that proof-facing structure (TASK-350).

- Aligned adjacent specs with the revised `SPEC-004` vocabulary by standardizing on `implicit control mailbox` in SPEC-013 and `Permit` as the canonical capability-verification allow decision in SPEC-017 (TASK-350).

- **Phase 52: Critical Contract Gap Remediation**
  - **TASK-322:** Implemented SPEC-024 compliant `capabilities:` syntax with declaration-site constraints
    - Changed `RoleDef` AST from `authority: Vec<Name>` to `capabilities: Vec<CapabilityDecl>`
    - Parser now supports `capabilities: [cap @ { constraints }]` syntax in role definitions
    - Type checker preserves constraints through capability composition
    - Runtime enforces constraints at capability invocation time
    - Lowering updated for implicit default role generation
    - All tests updated to use new syntax
  - **TASK-323:** Removed `--capability` CLI flag and updated SPEC-005
    - Capabilities now defined in Ash source files, libraries, or defaults only
    - CLI no longer accepts `--capability <name=uri>` argument
    - Supersedes TASK-317
  - **TASK-324:** Removed `--input` CLI flag and updated SPEC-005
    - Input parameters not yet supported via CLI (use `observe` or hardcoded values)
    - CLI no longer accepts `--input <json>` argument
    - Supersedes TASK-316
  - **TASK-325:** Fixed remaining clippy warnings
    - Fixed `redundant_closure` in `ash-engine/src/lib.rs:261`
    - Fixed `redundant_closure` in test file
    - Fixed `redundant_clone` in test file
    - Fixed `temporary_with_significant_drop` in e2e test
  - **TASK-326:** Updated SPEC-010 HTTP capability documentation
    - Added "4.3 Unimplemented Capabilities" section
    - Documented that `with_http_capabilities()` returns configuration error
    - Users directed to `with_custom_provider()` for HTTP implementation

- **Phase 54: Import Resolver Visibility Enforcement (single-crate model)**
  - **TASK-332:** Implemented `pub(crate)` enforcement in import resolver
    - Added `CrateId` tracking to `ModuleGraph` for future multi-crate support
    - `pub(crate)` now only allows imports within the same crate (same graph)
  - **TASK-333:** Implemented `pub(super)` enforcement in import resolver
    - Added parent tracking and `ancestors()` method to `ModuleGraph`
    - `pub(super)` now only allows imports from parent modules
  - **TASK-334:** Implemented `pub(in path)` enforcement in import resolver
    - Added `resolve_path()` and `is_descendant_or_same()` to `ModuleGraph`
    - `pub(in path)` now only allows imports from descendants of specified path
  - **TASK-335:** Added comprehensive visibility tests to import resolver
    - Added 49 visibility tests exceeding 25+ target
    - Added integration tests for real `.ash` file parsing
  - **TASK-343:** Fixed `pub(crate)` for real resolver path (regression fix)
    - Fixed issue where `set_crate()` was only called in tests
    - `pub(crate)` now works correctly with production resolver-built graphs
    - Note: True cross-crate enforcement is Phase 55 scope

- **Phase 55: Cross-Crate Boundary Enforcement**
  - **TASK-337:** Added crate root and dependency syntax
    - Parse `crate <name>;` declarations for crate identity
    - Parse `dependency <alias> from "<path>";` declarations
    - AST types: `CrateRootMetadata`, `DependencyDecl`
  - **TASK-338:** Extended `ModuleGraph` with crate identity
    - Added `CrateId` and `CrateInfo` types
    - Track module-to-crate ownership via `module_to_crate` mapping
    - Added `dependency_target()` for alias-to-crate resolution
  - **TASK-339:** Implemented dependency-aware multi-crate loading
    - `ModuleResolver` recursively loads dependency crates
    - Detects duplicate crate names, duplicate aliases, and dependency cycles
  - **TASK-340:** External import resolution and cross-crate visibility
    - Added `external::<alias>::...` path resolution
    - Only `pub` items visible across crate boundaries
    - `pub(crate)`, `pub(super)`, `pub(in path)` rejected for external imports
  - **TASK-341:** Aligned type checker with cross-crate visibility
    - Added `ModulePath::is_external()` and `crate_root()` methods
    - Type checker correctly distinguishes local vs external crate paths
    - Added multi-crate visibility regression tests

### Fixed

- TASK-310: Marked 3 failing cli_input_workflow_test tests as `#[ignore]` with known issue documentation
  - `test_multiple_workflow_parameters` - ignored: interpreter does not support String + Int concatenation
  - `test_boolean_workflow_parameter` - ignored: interpreter boolean to string conversion issue  
  - `test_list_workflow_parameter` - ignored: parser does not support `List<Int>` generic syntax in parameters
  - These are pre-existing limitations requiring significant interpreter/parser changes, out of scope for Phase 50

- TASK-288: `ash-repl` `:ast` now formats `ash_parser::surface::Expr` and `WorkflowDef` in the SPEC-011 structural shape, without synthetic workflow wrappers, spans, or debug-only internals.

- TASK-287: `ash-interp` now carries the active role in `Context.role_context`, enforces `Workflow::Oblig` and `Workflow::Check` against that runtime role context, and attributes `set`/`send` operations to the active role instead of the hardcoded `system` actor.

- TASK-286: `receive` now enforces capability-policy checks before non-blocking fallback and canonical stream-source selection, closing the runtime compliance gap with `observe`, `set`, and `send`.

- TASK-295: Preserve ADT qualified names (SPEC-003 Section 3.3 compliance)
  - `QualifiedName::parse()` now supports `::` separator for ADT naming conventions
  - `QualifiedName::display()` now uses `::` separator (e.g., `std::option::Option`)
  - Types with same root name in different modules are now distinct (e.g., `std::option::Option` ≠ `my::option::Option`)
  - Backward compatibility maintained for `.` separator
  - 8 new tests for qualified name parsing and equality

- TASK-296: Fix pub(super) visibility implementation (SPEC-009 compliance)
  - Changed `Visibility::Super` from unit variant to `Visibility::Super { levels: usize }`
  - This properly encodes parent-module semantics for restricted visibility
  - `levels` field indicates how many levels up (1 = parent, 2 = grandparent, etc.)
  - Added `ModulePath::ancestors()` method to support multi-level visibility checks
  - Updated `VisibilityExt::is_visible_path()` to use ancestor-based checking
  - Visibility checker now correctly restricts `pub(super)` to parent and its descendants
  - Parser updated to set `levels: 1` for `pub(super)` syntax
  - 30+ tests updated and passing for all visibility variants

- TASK-273: Fix `arb_pattern` binding name uniqueness in proptest_helpers
  - Added `PatternGenContext` to track used names during pattern generation
  - `arb_pattern_with_context()` generates unique sequential names (G_0, G_1, etc.)
  - Eliminated duplicate bindings between variables and rest patterns in lists
  - `test_arb_pattern_bindings_unique` property test now passes reliably
  - Removed inefficient `prop_filter` that was rejecting duplicate patterns

### Added

- **Phase 47: Spec Compliance Fixes (Post-46 Audit)**
  - **Critical Runtime Contract Fixes (47.1):**
    - TASK-274: Wire engine capability providers to RuntimeState
      - Added provider registry to RuntimeState with HashMap storage
      - Engine now passes configured providers during execution
      - Fixed Embedding API contract where providers were non-functional
      - 7 tests for provider wiring verification
    - TASK-275: Enable workflow obligation checking in type checker
      - Implemented ObligationCollector to walk AST and track obligations
      - Linear obligation tracking: oblige registers, check satisfies
      - Error types: UnsatisfiedObligations, UnknownObligation, ObligationAlreadySatisfied
      - 14 tests including property-based tests for obligation soundness
    - TASK-276: Fix unsound expression typing
      - Variable expressions now look up type from environment (not fresh type vars)
      - Implemented proper type inference for Block, Loop, For expressions
      - Added error types: UnboundVariable, NotIterable, UnsupportedExpression
      - 18 tests for type soundness verification
  - **Architecture Improvements:**
    - Type error variants now use `Box<Type>` to reduce stack size from ~200 bytes to ~64 bytes
    - Follows serde_json pattern for large error type handling
    - Documented in SPEC-003 Section 10 (Error Handling Conventions)
    - All clippy warnings resolved (clean build)
  - **High Priority CLI/REPL Fixes (47.2):**
    - TASK-277: REPL workflow definition storage
      - SessionState now stores workflows in HashMap<String, CompiledWorkflow>
      - Type checking occurs at definition time (fail-fast)
      - Support for workflow invocation by name in REPL session
      - 9 tests for workflow storage and invocation
    - TASK-278: Make CLI --input functional
      - JSON to Value conversion utilities (json_to_value, value_to_json)
      - Input binding to workflow parameters via --input flag
      - Validation of input against workflow signature
      - 12 tests for input functionality
    - TASK-279: Align CLI surface with SPEC-005
      - Proper exit codes: 2=parse, 3=type, 4=verification, 5=runtime, 6=I/O, 7=timeout
      - Global flags: --quiet, --color auto|always|never, repeatable -v
      - Command flags: --policy-check, --dry-run, --timeout, --capability
      - 22 tests for SPEC-005 compliance
  - **Medium Priority Compliance Fixes (47.3):**
    - TASK-280: Fix JSON output schema
      - Full SPEC-005 compliant JSON: schema_version, errors[], warnings[], timing{}, verification{}
      - Structured errors with severity, code, message, location, context, help
      - 13 tests for JSON schema compliance
    - TASK-281: Preserve ADT qualified names
      - AdtName struct with qualified, module, root fields
      - Same-name ADTs in different modules are distinct types
      - 19 tests for qualified name preservation
    - TASK-282: Fix pub(super) visibility
      - Proper ModulePath type with parent(), starts_with(), is_ancestor_of()
      - Correct "parent module and descendants" visibility checking
      - 20 tests for visibility compliance
    - TASK-283: Fix REPL multiline error detection
      - InputDetector with structural analysis for braces, strings
      - Distinguishes incomplete input from actual syntax errors
      - 16 tests for multiline detection

- **Phase 46: Unified Capability-Role Implementation (Partial)**
  - **Parser Extensions (46.1):**
    - TASK-259: Parse `plays role(R)` clause in workflow headers
    - TASK-260: Parse `capabilities: [...]` with `@ { constraints }` syntax
    - TASK-261: Lower capabilities to implicit `{workflow}_default` role
    - New AST types: RoleRef, CapabilityDecl, ConstraintBlock, ConstraintField, ConstraintValue
    - 67+ tests for parser extensions
  - **Type System Integration (46.2):**
    - TASK-262: RoleChecker validates role inclusion and composes capabilities
    - TASK-263: ConstraintChecker validates capability constraints against schema
    - TASK-264: EffectiveCapabilitySet merges capabilities from multiple sources
    - Type errors: UnknownRole, UnknownCapability, InvalidConstraintField, ConstraintTypeMismatch
    - 75+ tests for type system integration
  - **Runtime Integration (46.3):**
    - TASK-265: RoleRegistry resolves workflow roles to runtime capability grants
    - TASK-266: ConstraintEnforcer validates capability constraints at invocation time
    - TASK-267: YieldRouter routes `yield role(R)` to registered role handlers
    - Runtime types: RuntimeCapabilitySet, CapabilityGrant, PendingYield, ResumeResult
    - Error types: RoleError, CapabilityError, ConstraintViolation, YieldError
    - 70+ tests for runtime integration
  - **Agent Harness (46.4):**
    - TASK-268: Agent harness capability types for LLM agent integration
    - Types: AgentHarnessCapability, AgentHarnessConfig, AgentHarnessOperation
    - Security model: Permission-based with default deny on accept_response
    - Configuration: ProjectionPolicy, AcceptanceMode, max_retries, timeout_ms
    - 6 comprehensive tests for capability functionality
    - TASK-269: AgentHarness workflow pattern for LLM agent integration in ash-engine
    - Types: AgentHarness, HarnessError, HarnessResult
    - Operations: project_context, delegate_to_agent, validate_response, accept_response
    - 12 comprehensive tests for harness functionality
    - TASK-270: MCP (Model Context Protocol) capability provider for LLM communication
    - Types: McpProvider, McpConfig, McpCapabilities
    - Protocol: JSON-RPC 2.0 over HTTP with reqwest client
    - Operations: call (raw JSON-RPC), call_tool (MCP tools), get_prompt (MCP prompts)
    - Integration: Real MCP delegation in AgentHarness::delegate_to_agent
    - Testing: wiremock-based HTTP mocking for 4 integration tests

- **Reduced Syntax Specification (Phase 45)**
  - SPEC-024: Complete capability-role-workflow syntax specification with EBNF grammar (TASK-257)
  - DESIGN-014: Syntax reduction decision record documenting kept vs deferred features (TASK-257)
  - SPEC-017: Added Section 5 documenting constraint refinement syntax `@ { ... }` (TASK-258)
  - Deferred features: capability composition operators (`+`, `|`), use-site refinement, implicit role leak
  - Kept syntax: `plays role(R)`, `capabilities: [...]`, `capability @ { constraints }`

### Fixed

- TASK-285: Preserved proxy registry and suspended yield state across receive execution paths in `ash-interp`, so receive-driven proxy workflows can suspend and resume correctly through matched, wildcard, and control receive arms per SPEC-023.

- TASK-284: Preserved proxy workflow state across recursive execution paths in `ash-interp`, so nested `yield`/`proxy resume` flows now survive `let`, `if`, `observe`, `check`, and related control-flow wrappers per SPEC-023.

- **Code Quality Fixes (Phase 46 Follow-up)**
  - Fixed failing property test `prop_capability_with_multiple_params` by excluding reserved keywords from parameter name generation
  - Added missing reserved keywords to `is_keyword()`: `let`, `if`, `else`, `match`, `done`, `ret`, `yield`, `plays`, `capabilities`
  - Replaced `.unwrap()` with safe alternatives in `parse_workflow.rs` and `parse_pattern.rs` using `is_some_and()`/`is_none_or()`
  - **TASK-273: Fixed `arb_pattern()` binding name uniqueness**
    - Added `prop_filter` to ensure generated patterns have unique binding names
    - Prevents duplicate bindings when rest pattern (`G_`) matches a variable name (`G_`) in the same record
    - Test `test_arb_pattern_bindings_unique` now passes consistently
  - Added `#[must_use]` to Result-returning functions per rust-skills guidelines:
    - `RoleRegistry::resolve_workflow_roles()`
    - `RuntimeCapabilitySet::check_use()`
    - `ConstraintEnforcer::check()`
    - `YieldRouter::route_yield()`
    - `YieldRouter::resume_with_response()`

- **Stale Documentation Update (TASK-255)**
  - Fixed `README.md` example reference from non-existent `examples/multi_agent.ash` to `examples/multi_agent_research.ash`
  - Fixed `docs/API.md` syntax error: `pubuse provenance::*;` → `pub use provenance::*;`
  - Updated `docs/spec/README.md` with correct spec file mappings matching actual SPEC files

### Added

- **Trace Flags Implementation (TASK-254)**
  - Implemented `--lineage` flag to include data lineage information in trace output
  - Implemented `--verify` flag to compute and include integrity verification data (Merkle tree root hash) in trace output
  - Added `IntegrityData` struct for trace integrity metadata
  - Extended `TraceResult` with optional `lineage` and `integrity` fields
  - Added 3 new tests for lineage and integrity flag functionality

- **EngineBuilder Methods Implementation (TASK-246)**
  - Added `with_http_capabilities(config)` method that returns a configuration error with guidance to use `with_custom_provider()` instead. Native HTTP provider implementation is planned for a future release.
  - Implemented `with_custom_provider(name, provider)` to register custom capability providers that can extend or override built-in providers
  - Added `HttpConfig` struct for HTTP capability configuration (for future use)
  - Updated `Engine` to store registered providers (wired for future execution integration)
  - Added 10 new tests covering HTTP capabilities, custom providers, and combined builder configuration

- **Float Handling with Explicit Errors (TASK-253)**
  - Added `LoweringError::FloatNotSupported` variant for explicit float rejection
  - Lowering functions now return `Result` types for proper error propagation
  - JSON float handling in CLI now returns clear error instead of silent Null

- **Provider Implementations (TASK-247)**
  - Implemented `StdioProvider` with real stdio operations (print, println, read_line)
  - Implemented `FsProvider` with real filesystem operations (exists, read_file, write_file)
  - Added `FsConfig` for capability constraints (allowed_paths, read_only, base_dir)
  - Added 43 comprehensive tests for provider functionality

- **Workflow::CheckObligation Execution (TASK-241)**
  - Implemented runtime execution for `Workflow::CheckObligation` per SPEC-022
  - Discharges obligations and returns boolean result
  - Integrated with linear obligation tracking in Context

- **Yield Placeholder Replacement (TASK-242)**
  - Replaced `Yield` placeholder lowering with real implementation
  - Added `lower_type_to_type_expr()` and `lower_yield_arms()` helper functions
  - Added 7 comprehensive lowering tests

- **YIELD Runtime Execution (TASK-243)**
  - Implemented `ExecError::YieldSuspended` variant with full yield context
  - Yield now evaluates request expression and creates proper suspension
  - Added `yield_execution_tests.rs` with 6 integration tests

- **PROXY_RESUME Runtime (TASK-244)**
  - Implemented full PROXY_RESUME workflow execution
  - Added `resume_var` field to `YieldState` for response binding
  - Resumes suspended yields by correlation_id with continuation binding

- **Workflow::Oblige Execution (TASK-240)**
  - Implemented runtime execution for `Workflow::Oblige` to satisfy SPEC-022 contract requirements
  - Obligations are now tracked in the runtime `Context` with linearity checking (duplicate oblige fails)
  - `CheckObligation` discharges obligations and returns boolean indicating success
  - Added 15 integration tests in `crates/ash-interp/tests/obligation_execution_tests.rs`

- Comprehensive workspace audit for 2026-03-26 in `docs/audit/codex-comprehensive-review.md`. The report captures current spec-compliance gaps, tooling failures, security observations, and a prioritized remediation list for the live Rust workspace.

- **Workflow Contracts with Linear Obligation Tracking (Phase 37, SPEC-022)**
  - Hoare-style workflow contracts with `requires` and `ensures` clauses
  - Linear obligation tracking: `oblige obligation_name` creates, `check obligation_name` discharges
  - Requirement checking with capabilities (`HasCapability`), roles (`HasRole`), and arithmetic constraints
  - SMT-based arithmetic constraint checking using Z3 for symbolic verification
  - Audit trail integration with JSON Lines format for obligation checks
  - Branch/parallel obligation discharge semantics via set intersection
  - 600+ new tests covering obligations, requirements, and contract parsing
  - Canonical SPEC-022 documentation in `docs/spec/` (TASK-226 through TASK-232)

- Full parametric polymorphism (generics) for Ash type system. Type constructors like `Option<Int>` and `Option<String>` are now distinct, distinguishable types. (TASK-127, TASK-128, TASK-129, TASK-130)
- `Type::Constructor` variant with `QualifiedName`, type arguments, and `Kind` annotation for future higher-kinded type support.
- `Kind` system for classifying type constructors (`*`, `* -> *`, etc.).
- `QualifiedName` for module-qualified type names.
- Iso-recursive type unfolding for generic field access and pattern matching.
- Pattern typing and exhaustiveness checking for generic constructors.
- Property-based tests for unification soundness, reflexivity, and symmetry.

### Changed

- `type_expr_to_type` now properly converts `TypeExpr::Constructor` to `Type::Constructor` instead of losing constructor information.
- `build_constructor_type` now returns the constructor type (e.g., `Option<T>`) instead of just the type parameter.
- Type alias expansion now properly unfolds to underlying types.

### Code Quality

- Fixed clippy warnings across workspace (TASK-249)
  - Fixed dead_code warnings in test files
  - Fixed redundant clone warnings
  - All files now pass `clippy -D warnings`

- Fixed unexpected_cfgs warning in ash-typeck (TASK-252)
  - Removed empty `proptest` feature from Cargo.toml
  - Simplified cfg condition to `#[cfg(test)]`

- Formatted all code with `cargo fmt` (TASK-250)
- Fixed all rustdoc warnings for clean documentation generation (TASK-251)
  - Fixed broken intra-doc links
  - Fixed invalid code blocks
  - Fixed invalid HTML tags in doc comments

### Fixed

- **Role Obligation Discharge (TASK-248)**
  - Fixed `RoleContext::discharge()` to verify obligations are declared on the role before discharge
  - Added `DischargeError` enum with `UndeclaredObligation` and `AlreadyDischarged` variants
  - Changed return type from `bool` to `Result<(), DischargeError>` for proper error handling
  - Updated all tests to use the new Result-based API

- **SmtContext Thread Safety (TASK-245)**
  - Removed unsound `unsafe impl Send/Sync for SmtContext`
  - Added `PhantomData<Rc<()>>` to enforce `!Send` and `!Sync` at compile time
  - Documented that `SmtContext` must be created and used on a single thread only
  - For multi-threaded use, create a separate `SmtContext` per thread

- `Option<Int>` and `Option<String>` no longer incorrectly unify.
- Error messages now show readable type names (`Option<Int>`) instead of internal variable IDs (`Var<42>`).
- Fixed Type Expression Conversion (TypeEnv). Replaced stubbed `TypeExpr::Constructor` handling that lost constructor information. `type_expr_to_type` now properly converts constructor names and all arguments, type alias expansion now resolves to underlying types, and name resolution is available via the new `resolve_type` helper.
- Cleaned up documentation in `kind.rs` to avoid unnecessary `ignore` attributes on code blocks.

### Added

- Role-convergence design and planning scaffold for TASK-216 through TASK-220. `docs/plans/2026-03-23-role-contract-simplification-design.md` now records the simplified role model, `docs/plans/2026-03-23-role-convergence-implementation-plan.md` turns that design into an implementation sequence, and `docs/plan/PLAN-INDEX.md` plus TASK-216 through TASK-220 now track the follow-up parser/core/runtime/example work needed to remove legacy role-supervision residue.
- Follow-up blocker-remediation planning for the remaining role-convergence gaps after TASK-220. `docs/plans/2026-03-23-role-convergence-blocker-remediation-design.md` now records the narrowed design for replacing placeholder role-obligation lowering and reconciling touched docs/examples with the canonical surface, while `docs/plans/2026-03-23-role-convergence-blocker-remediation-plan.md`, `docs/plan/PLAN-INDEX.md`, and TASK-221 through TASK-224 break that work into focused self-contained implementation tasks.

### Changed

- Inline-module parser honesty follow-up now rejects unsupported canonical inline items such as `workflow`, `policy`, `datatype`, and visibility-qualified entries explicitly even after recovery from earlier unknown items instead of skipping them silently, while the module role-lowering helper surface is narrowed to the maintained test-only crate-internal path (TASK-225).
- Review-driven role-convergence wording cleanup now removes stale placeholder-lowering wording from TASK-218 and makes the closeout audit explicit that module role lowering remains a maintained test-only helper surface rather than a general parser-facing lowering API (TASK-218, TASK-225).
- Phase 36 role-convergence closeout now includes a fresh audit note and reconciled task bookkeeping. `docs/audit/2026-03-23-role-convergence-closeout-audit.md` records the post-TASK-221 through TASK-225 evidence, distinguishes intentional historical/process-supervision references from live role syntax, and marks the blocker-remediation phase complete (TASK-224, TASK-225).
- Touched role docs and examples now use honest canonical/reference framing: tutorial and appendix guidance now point readers back to `docs/spec/` for the canonical syntax contract, scenario examples are explicitly marked as reference-oriented where they are not conformance samples, and the multi-agent research example no longer refers to an undefined `reviewer` role (TASK-223).
- Parsed inline-module `role` definitions now lower through regression-covered test-only crate-internal parser/module helpers, so named role obligations flow into the core `RoleObligationRef` carrier through the maintained module helper path, same-module capability definitions preserve authority metadata during role lowering, and unsupported canonical inline definitions are rejected explicitly instead of being skipped silently (TASK-222).
- Core role metadata now preserves named role-obligation references with a dedicated `RoleObligationRef` carrier instead of reusing workflow `Obligation` semantics for identifier-only role obligations (TASK-221).
- Examples and residual user-facing docs now consistently reflect the simplified flat role contract, removing canonical `supervises` usage from touched role examples, updating approval examples to use explicit named-role syntax, and adding a focused role-convergence audit note for the remaining intentional historical/process-supervision references (TASK-220).
- Runtime approval-role handling now explicitly documents and tests the flat named-role contract already used by `ash-interp`, ensuring `RequireApproval` outcomes preserve the named approval role directly without implying supervision or inherited hierarchy semantics (TASK-219).
- Inline module parsing now recognizes source `role` definitions in inline modules, preserving named role authorities and named role obligations in the surface AST and lowering them into the simplified core role carrier shape through the maintained test-only crate-internal module helper path (TASK-218).
- Removed the legacy `supervises` role field from parser and core role structures, dropped placeholder lowering that manufactured empty supervision data, and returned `supervises` to ordinary identifier handling in parser contexts (TASK-217).
- Canonical role contracts no longer treat supervision as part of the role model (TASK-216). `SPEC-002` now defines `role_def` with authority and obligations only, `SPEC-001` now defines the matching core role shape without `supervises`, and `SPEC-017` / `SPEC-018` now clarify that approval-role references remain flat named-role policy/verification constructs rather than hierarchy-derived supervision.

- `Expr::Match` exhaustiveness checking in `ash-typeck` (`check_expr`) for enum scrutinees resolved via constructor or variant patterns, reporting `ConstructorError::NonExhaustiveMatch` when arms omit variants (TASK-130).
- Completed ADT interpreter convergence for constructor evaluation, pattern matching, and match/if-let behavior (TASK-131, TASK-132, TASK-133). `ash-interp` now evaluates receive/mailbox patterns through the shared `match_pattern` engine (including variants), and explicit Option-style match/if-let runtime tests lock expected binding/branch semantics.
- Control-link retention policy handoff for TASK-212. `docs/reference/control-link-retention-policy.md` now freezes retained tombstones as runtime-state-owned terminal visibility, `SPEC-004` / `SPEC-021` now encode the same observable semantics, and the related design notes now point to the canonical retention contract.
- Residual spec-audit follow-up closeout now uses fully consistent historical framing. The final convergence audit summary now matches the Phase 34 addendum, and the Phase 34 plan is explicitly marked complete rather than reading like a still-live execution plan.
- Residual spec hygiene closeout for TASK-215. `SPEC-015` now uses canonical `Int` examples in the remaining typed-provider snippets, and the final convergence audit now records that the Phase 34 spec-only findings are closed while keeping `TASK-212` as the remaining non-blocking follow-up.
- Residual policy and typed-provider spec drift cleanup for TASK-214. `SPEC-007` now uses a genuinely contradictory SMT example, `SPEC-015` no longer forwards schema-first code generation to the unrelated `SPEC-016` output spec, and `SPEC-010` / `SPEC-016` now explicitly keep provider effect granularity at the embedding boundary without widening runtime scope.
- TASK-213 now reconciles the module/import spec scope. `SPEC-009` now defers `use` and `pub use` to `SPEC-012` instead of treating them as future module features, and the touched examples now use canonical type names.
- Residual spec-audit follow-up plan and task set for the remaining docs-only findings after TASK-176. `docs/plan/2026-03-20-residual-spec-audit-follow-up-plan.md` now defines the bounded post-convergence docs phase, and `TASK-213` through `TASK-215` now cover the remaining module/import scope conflict, typed-provider/policy example drift, and low-severity spec hygiene cleanup.
- Final convergence closeout audit for TASK-176. `docs/audit/2026-03-20-final-convergence-audit.md` now records the closure matrix for the original implementation drift classes, confirms repository-wide verification, and makes both `TASK-212` and the remaining spec-only documentation debt explicit rather than leaving any convergence gap implicit.
- Canonical ADT stdlib and example surface for TASK-175. `std/src/prelude.ash` now exposes the full canonical Option/Result helper surface, `examples/README.md` documents the same surface for readers, and parser-level stdlib-surface tests lock the contract in.
- Canonical REPL authority and tooling-observable convergence for TASK-172, TASK-173, and TASK-208. `ash-repl` now exports the canonical REPL command surface and session configuration used by both REPL entrypoints, REPL `:type` reporting now flows through the canonical parse/type-check pipeline with focused expression inference support, and `ash run` / `ash trace` now emit contract-aligned observable output with focused CLI regression coverage.
- Follow-up task for long-term `ControlLink` retention design. [TASK-212](docs/plan/tasks/TASK-212-design-control-link-retention-policy.md) now tracks the bounded-retention/cleanup design for terminated supervision state after `TASK-206` freezes tombstone retention as the current runtime behavior.
- Runtime-verification input contract and follow-up task for the capability-versus-obligation split. [docs/reference/runtime-verification-input-contract.md](docs/reference/runtime-verification-input-contract.md) now freezes the distinction between workflow capability declarations and obligation-backed runtime requirements, and [TASK-209](docs/plan/tasks/TASK-209-separate-runtime-verification-input-classes.md) now blocks [TASK-170](docs/plan/tasks/TASK-170-implement-end-to-end-receive-execution.md) and [TASK-171](docs/plan/tasks/TASK-171-align-runtime-policy-outcomes.md) until aggregate verification exposes those inputs separately.
- TASK-206 now explicitly carries the follow-up for the transitional control-link registry introduced by TASK-205. [docs/plan/tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md](docs/plan/tasks/TASK-206-align-runtime-admission-rejection-and-commitment-visibility.md) and [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now require replacing the temporary shared process-global control registry with explicit runtime-owned lifecycle state and a defined cleanup versus tombstone policy for terminated instances.
- Explicit execution-order bridge notes for the old and new convergence tasks. [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) and [TASK-170](docs/plan/tasks/TASK-170-implement-end-to-end-receive-execution.md), [TASK-171](docs/plan/tasks/TASK-171-align-runtime-policy-outcomes.md), [TASK-172](docs/plan/tasks/TASK-172-unify-repl-implementation.md), [TASK-173](docs/plan/tasks/TASK-173-implement-repl-type-reporting.md), and [TASK-176](docs/plan/tasks/TASK-176-final-convergence-audit.md) now make the downstream relationship to TASK-205 through TASK-208 explicit so the original convergence phases and the new runtime/tooling implementation phases read as one ordered execution path.
- Runtime-boundary implementation plan and task set for TASK-205 through TASK-207. [docs/plan/2026-03-20-runtime-boundary-implementation-plan.md](docs/plan/2026-03-20-runtime-boundary-implementation-plan.md) now turns the runtime steering brief into concrete runtime-first implementation work, and [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now tracks the new runtime execution completeness, runtime boundary visibility, and trace/provenance hardening tasks.
- Tooling observable convergence plan and CLI output task for TASK-208. [docs/plan/2026-03-20-tooling-observable-convergence-plan.md](docs/plan/2026-03-20-tooling-observable-convergence-plan.md) now maps the tooling steering brief onto the minimum-risk implementation path by reusing [TASK-172](docs/plan/tasks/TASK-172-unify-repl-implementation.md) and [TASK-173](docs/plan/tasks/TASK-173-implement-repl-type-reporting.md) and adding [TASK-208](docs/plan/tasks/TASK-208-align-cli-run-and-trace-observable-output.md) for CLI `run` / `trace` output convergence while deferring the optional stage-guidance overlay.
- Tooling and surface steering brief for TASK-204. [docs/plan/2026-03-20-tooling-surface-steering-brief.md](docs/plan/2026-03-20-tooling-surface-steering-brief.md) now merges the CLI/REPL and trace-presentation audits into one review artifact, defines later tooling clusters around REPL observable-behavior convergence, CLI run/trace output convergence, and presentation-only stage-guidance overlays, and keeps projection and runtime semantic authority out of the tooling phase.
- Trace export and presentation audit for TASK-203. [docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md](docs/audit/2026-03-20-trace-export-and-presentation-planning-review.md) now classifies the CLI trace command, provenance recorder, and export helpers as runtime-only, and [docs/plan/tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md](docs/plan/tasks/TASK-203-audit-trace-export-and-presentation-surfaces.md) / [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now mark the task complete while leaving stage-aware wording to later tooling/surface planning.
- CLI and REPL interaction-planning audit for TASK-202. [docs/audit/2026-03-20-cli-and-repl-interaction-planning-review.md](docs/audit/2026-03-20-cli-and-repl-interaction-planning-review.md) now classifies `ash run`, `ash trace`, REPL command handling, and inspection surfaces as runtime-observable, keeps explanatory stage guidance separate, and records the remaining `:type` wording cleanup as presentation-level convergence for later tooling planning.
- Runtime-boundary steering brief for TASK-201. [docs/plan/2026-03-20-runtime-boundary-steering-brief.md](docs/plan/2026-03-20-runtime-boundary-steering-brief.md) now merges the runtime execution and trace/provenance audits into one review artifact, defines later runtime task clusters around runtime completeness, acceptance/commitment visibility, and trace/provenance hardening, and keeps tooling and interaction concerns out of the runtime-boundary phase.
- Runtime execution boundaries audit for TASK-199. [docs/audit/2026-03-20-runtime-execution-boundaries-interaction-planning-review.md](docs/audit/2026-03-20-runtime-execution-boundaries-interaction-planning-review.md) now classifies the engine, interpreter, observation, policy, and effectful commit surfaces as runtime-only, with the remaining work identified as runtime completeness rather than runtime/reasoner overlap.
- Runtime trace and provenance planning review for TASK-200. [docs/audit/2026-03-20-runtime-trace-and-provenance-planning-review.md](docs/audit/2026-03-20-runtime-trace-and-provenance-planning-review.md) now confirms the trace recorder, trace events, export helpers, and workflow-wrapper surfaces remain runtime-only, and [docs/plan/tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md](docs/plan/tasks/TASK-200-audit-runtime-trace-and-provenance-surfaces.md) / [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now mark the task complete for later runtime-boundary synthesis.
- Runtime-boundary and tooling/surface implementation-planning scaffolds for TASK-199 through TASK-204. [docs/plan/2026-03-20-runtime-boundary-implementation-planning-plan.md](docs/plan/2026-03-20-runtime-boundary-implementation-planning-plan.md) and [docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md](docs/plan/2026-03-20-tooling-surface-implementation-planning-plan.md) now define the next two review-gated planning phases, while [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now tracks the new runtime-boundary and tooling/surface tasks and their phase-end steering briefs before any new code-facing work opens.
- Revised runtime-reasoner convergence map for TASK-198. [docs/plan/2026-03-20-runtime-reasoner-revised-convergence-map.md](docs/plan/2026-03-20-runtime-reasoner-revised-convergence-map.md) now records that TASK-164 through TASK-171 remain unchanged, TASK-172 and TASK-173 only need in-place reference updates, and later code-facing work should be split into separate runtime, tooling, and provenance/trace clusters.
- Runtime-reasoner implementation-planning impact audit for TASK-196. [docs/audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md](docs/audit/2026-03-20-planned-convergence-tasks-runtime-reasoner-impact-review.md) now classifies TASK-164 through TASK-173 against the new runtime-reasoner corpus, confirming the parser/lowering/type/runtime tasks are unchanged and the REPL tasks need only reference updates rather than scope changes.
- Runtime-reasoner implementation-planning scaffold for TASK-196 through TASK-198. [docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md](docs/plan/2026-03-20-runtime-reasoner-implementation-planning-plan.md) now defines the next docs/planning phase after the runtime-reasoner spec handoff, and [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now tracks the impact audit, implementation-planning-surface note, and revised convergence-map synthesis tasks needed before opening new code-facing work.
- Runtime-reasoner spec handoff for TASK-195. [docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md](docs/plan/2026-03-20-runtime-reasoner-spec-handoff.md) now closes the docs-only follow-up phase by listing the authoritative interaction-facing docs, restating protected runtime-only areas, and defining the boundary for later implementation planning without creating implementation tasks yet.
- Human-facing surface guidance boundary for TASK-194. [docs/reference/surface-guidance-boundary.md](docs/reference/surface-guidance-boundary.md) now states that advisory/gated/committed stage guidance belongs in explanatory documentation first, not new surface syntax, and explicitly protects `exposes`, monitor views, and other runtime-only constructs from being reused as stage markers.
- Projection and monitorability terminology for TASK-193. [docs/design/LANGUAGE-TERMINOLOGY.md](docs/design/LANGUAGE-TERMINOLOGY.md) now reserves `projection`, `monitorability`, and `exposed workflow view` as distinct terms, constrains `observe` to workflow input acquisition, and [docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md](docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md) now states explicitly that runtime visibility is separate from reasoner projection.
- Runtime authority framing for TASK-192. [docs/spec/SPEC-004-SEMANTICS.md](/home/dikini/Projects/ash/docs/spec/SPEC-004-SEMANTICS.md) now states that authoritative runtime state, validation, rejection, commitment, trace, and provenance remain runtime-owned, while external reasoner outputs remain advisory until accepted under separate interaction contracts.
- Runtime-to-reasoner interaction contract for TASK-191. [docs/reference/runtime-to-reasoner-interaction-contract.md](docs/reference/runtime-to-reasoner-interaction-contract.md) now defines injected context, advisory outputs, acceptance boundaries, runtime-owned commitment, and the explicit non-overlap between projection and runtime-only constructs such as monitor views, `exposes`, workflow observability, and `MonitorLink`.
- Runtime-reasoner spec follow-up planning scaffold for TASK-191 through TASK-195. [docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md](docs/plan/2026-03-20-runtime-reasoner-spec-follow-up-plan.md) now defines the docs-only follow-up phase after the runtime-reasoner design review, and [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now tracks the new phase and its tasks for the interaction contract, `SPEC-004` framing, terminology tightening, surface-guidance boundary, and final handoff synthesis.
- Runtime-reasoner audit reports and delta program for TASK-188 through TASK-190. [docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md](docs/audit/2026-03-20-runtime-and-verification-reasoner-boundaries-review.md) and [docs/audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md](docs/audit/2026-03-20-surface-and-observability-reasoner-boundaries-review.md) now record the runtime-only versus interaction-layer audit outcome, and [docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md](docs/plan/2026-03-20-runtime-reasoner-spec-delta-program.md) now orders the follow-up work so projection and advisory interaction are added without overloading monitors, `exposes`, workflow observability, or other runtime-only contracts.
- Runtime-reasoner separation rules for TASK-187. [docs/reference/runtime-reasoner-separation-rules.md](docs/reference/runtime-reasoner-separation-rules.md) now freezes the “does this make sense without a reasoner present?” test, defines runtime-only versus interaction-layer versus split concerns, and explicitly keeps monitor views, `exposes`, and workflow observability out of reasoner-projection semantics.
- Runtime-reasoner design-review planning scaffold for TASK-187 through TASK-190. [docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md](docs/design/RUNTIME_REASONER_INTERACTION_MODEL.md) now has a matching review phase in [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md), plus a design-review plan in [docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md](docs/plan/2026-03-20-runtime-reasoner-design-review-plan.md) and task definitions for freezing separation rules, auditing canonical docs, and synthesizing the follow-up spec delta program.
- Monitor authority and exposed workflow views for TASK-186. [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md), and [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) now define an explicit `exposes { ... }` workflow clause, first-class `MonitorLink` authority, exposed monitor views, and monitor-view observability without adding a monitor-specific policy sublanguage.
- Spec hardening readiness audit for TASK-184. [docs/audit/2026-03-19-spec-hardening-readiness-review.md](docs/audit/2026-03-19-spec-hardening-readiness-review.md) now gates Rust convergence, confirms Lean formalization has a stable starting corpus, and records that the hardened language definition has no canonical `catch`.
- TASK-183 follow-up refinement for the formalization boundary. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now distinguishes the canonical semantic corpus from authoritative source/handoff contracts and historical artifacts, and [docs/spec/SPEC-046-LEAN-REFERENCE.md](docs/spec/SPEC-046-LEAN-REFERENCE.md) is explicitly marked as a legacy sketch rather than a competing current spec.
- Formalization boundary note for TASK-183. [docs/reference/formalization-boundary.md](docs/reference/formalization-boundary.md) now names the canonical Lean/Rust proof corpus, separates migration-only artifacts, and lists the initial proof and bisimulation targets for the hardened language contract.
- TASK-182 follow-up tightening for runtime observable behavior. [SPEC-011](docs/spec/SPEC-011-REPL.md) now defers REPL error rendering to [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) now treats verification warnings as observable tooling output, and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) is now mechanically a handoff note rather than a second canonical owner.
- Runtime observable behavior specification for TASK-182. [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) now owns the canonical CLI/REPL observable contract, runtime verification visibility, constructor-shaped ADT display, and explicit `Result`-based recoverable failure handling.
- ADT dynamic semantics tightening for TASK-181. [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md), [docs/reference/parser-to-core-lowering-contract.md](docs/reference/parser-to-core-lowering-contract.md), [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md), and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) now define canonical constructor evaluation, constructor-shaped runtime `Variant` values, `Match` no-match behavior, and `if let` as sugar for `match` with a wildcard fallback arm. SPEC-004 now carries the normative operational semantics directly.
- Follow-up tightening for TASK-180. [SPEC-006](docs/spec/SPEC-006-POLICY-DEFINITIONS.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-018](docs/spec/SPEC-018-CAPABILITY-MATRIX.md), and [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) now require named policy bindings at capability sites and define the capability-verification outcome set as a verification-time interface with explicit pre-execution incompatibility rejection for unsupported approval or transformation outcomes.

### Fixed

- Runtime trace and provenance boundaries now use one canonical wrapper framing path (TASK-207).
  `ash-provenance` now exposes a `WorkflowTraceSession` that records `started` on entry and
  terminal `completed` on exit, failed runs now record `error` before `completed(false)`, and the
  current CLI trace wrappers plus `#[workflow]` macro now route through that same runtime-only
  session API. `ash-macros` also now has integration coverage for the downstream expansion path.
- Aligned ADT match exhaustiveness checking with runtime variant field-shape semantics: unit-variant patterns now cover only zero-field variants (TASK-130).
- Updated parser pattern syntax so bare uppercase constructor identifiers like `None` are parsed as unit variant patterns instead of variable bindings (TASK-130).
- `TASK-206` now makes the current terminated-control retention behavior explicit and tests it directly. `ash-interp` stateful runtime-boundary tests now lock in that killed control links remain observable as terminated tombstones across later executions sharing the same `RuntimeState`.
- Cleared the remaining workspace clippy warnings so the repository-level CI gate is clean again (TASK-210). `ash-core` test construction now uses `Box::default()` instead of boxing an empty vector directly, and `ash-repl` test ANSI stripping now iterates with `for ... in chars.by_ref()` so `cargo clippy --all-targets --all-features` and `cargo test --all` both pass on the merged codebase.

### Added (continued 1)

- Added a control-authority contract revision gate before the runtime hardening batch (TASK-211). [docs/plan/tasks/TASK-211-revise-control-link-authority-contract.md](docs/plan/tasks/TASK-211-revise-control-link-authority-contract.md) now freezes the required documentation work to revise `ControlLink` from affine one-shot control to reusable supervision authority, and [TASK-205](docs/plan/tasks/TASK-205-implement-runtime-action-and-control-link-execution.md) is now explicitly blocked on that contract update.

### Changed (continued 1)

- Revised the canonical control-link contract from affine one-shot control to reusable supervision authority (TASK-211). [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md), [SPEC-021](docs/spec/SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), and the related design/reference notes now define `ControlLink` as reusable for non-terminal supervision operations, with terminal invalidation driven by runtime instance state rather than unconditional first-use consumption.
- Removal of `attempt`/`catch` from the canonical language for TASK-185. [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-014](docs/spec/SPEC-014-BEHAVIOURS.md), [SPEC-016](docs/spec/SPEC-016-OUTPUT.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), and [SPEC-020](docs/spec/SPEC-020-ADT-TYPES.md) now require explicit `Result` values and pattern matching for recoverable failures.
- Policy evaluation and verification semantics tightening for TASK-180. [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-006](docs/spec/SPEC-006-POLICY-DEFINITIONS.md), [SPEC-007](docs/spec/SPEC-007-POLICY-COMBINATORS.md), [SPEC-008](docs/spec/SPEC-008-DYNAMIC-POLICIES.md), [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md), [SPEC-018](docs/spec/SPEC-018-CAPABILITY-MATRIX.md), and [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) now define one policy story from named binding through lowered `CorePolicy` to runtime `PolicyDecision`, with workflow `decide` limited to `Permit` / `Deny` and capability verification using the richer verification outcome set.
- Receive mailbox and scheduling semantics formalization for TASK-179. [SPEC-002](docs/spec/SPEC-002-SURFACE.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), [SPEC-013](docs/spec/SPEC-013-STREAMS.md), and [SPEC-017](docs/spec/SPEC-017-CAPABILITY-INTEGRATION.md) now define the source-selection model, source scheduling modifier semantics, guard timing, consumption timing, global `_` fallback, and one timeout budget for `receive`.
- Phase-judgment and rejection-boundary tightening for TASK-178. [SPEC-001](docs/spec/SPEC-001-IR.md), [SPEC-003](docs/spec/SPEC-003-TYPE-SYSTEM.md), [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md), and the canonical reference docs now separate parser, lowering, type, and runtime rejection classes from contract text while leaving implementation drift in task/planning notes.
- Canonical core language and execution-neutral IR tightening for TASK-177. [SPEC-001](docs/spec/SPEC-001-IR.md), [SPEC-002](docs/spec/SPEC-002-SURFACE.md), and [SPEC-004](docs/spec/SPEC-004-SEMANTICS.md) now state the core-language form set, surface-sugar boundary, and backend-neutral IR invariants explicitly so later Rust and Lean work can treat them as canonical contract.
- Spec-hardening design in [docs/plan/2026-03-19-spec-hardening-design.md](docs/plan/2026-03-19-spec-hardening-design.md) and implementation plan in [docs/plan/2026-03-19-spec-hardening-plan.md](docs/plan/2026-03-19-spec-hardening-plan.md). These define the documentation gate required before Rust convergence resumes, with explicit goals for unambiguous Rust/Lean implementation, execution-neutral IR, and theory-grounded semantics.
- Spec-hardening task files [TASK-177](docs/plan/tasks/TASK-177-freeze-canonical-core-language-and-ir.md) through [TASK-184](docs/plan/tasks/TASK-184-audit-spec-hardening-readiness.md). These add a new pre-alignment task track for canonical core semantics, phase judgments, `receive`, policy, ADT, observable-behavior, and formalization-boundary tightening.
- [docs/reference/type-to-runtime-contract.md](docs/reference/type-to-runtime-contract.md) and [docs/reference/runtime-observable-behavior-contract.md](docs/reference/runtime-observable-behavior-contract.md) as the canonical type/runtime and runtime/observable handoff references (TASK-163). They freeze required type-layer outputs, runtime/verification rejection boundaries, normative REPL-observable behavior, and stdlib-visible ADT/runtime guarantees for downstream convergence work.
- [docs/reference/parser-to-core-lowering-contract.md](docs/reference/parser-to-core-lowering-contract.md) as the canonical lowering handoff for stabilized workflow, policy, `receive`, and ADT forms (TASK-162). It defines the required surface-to-core mappings, lowering-time rejection cases, and preservation rules for downstream parser/core convergence work.
- [docs/reference/surface-to-parser-contract.md](docs/reference/surface-to-parser-contract.md) as the canonical parser handoff for stabilized workflow, policy, and ADT forms (TASK-161). It fixes the accepted syntax, required surface AST outputs, legal parser rejections, and the parser-versus-later-phase boundary for downstream convergence work.
- Convergence continuation task files [TASK-161](docs/plan/tasks/TASK-161-surface-to-parser-handoff-contract.md) through [TASK-176](docs/plan/tasks/TASK-176-final-convergence-audit.md). These extend the spec-to-implementation convergence program with explicit handoff-reference, parser/lowering, type/runtime, REPL/CLI, ADT, and final-audit tasks.
- [docs/design/LANGUAGE-TERMINOLOGY.md](docs/design/LANGUAGE-TERMINOLOGY.md) as a shared language guide for project documents. It standardizes terms such as `source scheduling modifier`, `scheduler`, `InstanceAddr`, and `ControlLink`, and reserves `policy` for authorization semantics.
- Phase-A convergence task files in [docs/plan/tasks/TASK-156-canonicalize-workflow-form-contracts.md](docs/plan/tasks/TASK-156-canonicalize-workflow-form-contracts.md), [docs/plan/tasks/TASK-157-canonicalize-policy-contracts.md](docs/plan/tasks/TASK-157-canonicalize-policy-contracts.md), [docs/plan/tasks/TASK-158-canonicalize-streams-runtime-verification-contracts.md](docs/plan/tasks/TASK-158-canonicalize-streams-runtime-verification-contracts.md), [docs/plan/tasks/TASK-159-canonicalize-repl-cli-contracts.md](docs/plan/tasks/TASK-159-canonicalize-repl-cli-contracts.md), and [docs/plan/tasks/TASK-160-canonicalize-adt-contracts.md](docs/plan/tasks/TASK-160-canonicalize-adt-contracts.md). Splits the first convergence phase into concrete documentation tasks with explicit requirements, TDD-style review steps, dependencies, and non-goals.
- Spec-to-implementation convergence design in [docs/plan/2026-03-19-spec-to-implementation-convergence-design.md](docs/plan/2026-03-19-spec-to-implementation-convergence-design.md). Defines the spec-first recovery model, phase ordering, task-shaping rules, and completion criteria for bringing Rust code back into compliance.
- Spec-to-implementation convergence plan in [docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md](docs/plan/2026-03-19-spec-to-implementation-convergence-plan.md). Breaks convergence into fresh follow-up tasks ordered from canonical spec repair through final implementation audit.
- Rust codebase review findings report in [docs/audit/2026-03-19-rust-codebase-review-findings.md](docs/audit/2026-03-19-rust-codebase-review-findings.md). Records checklist-driven implementation findings across baseline, policy, REPL/CLI, streams/runtime-verification, and ADT clusters without modifying Rust source.
- Rust codebase review checklist in [docs/audit/2026-03-19-rust-codebase-review-checklist.md](docs/audit/2026-03-19-rust-codebase-review-checklist.md). Maps audit-identified risky task clusters to concrete Rust review targets and questions.
- Non-Lean task consistency audit report in [docs/audit/2026-03-19-task-consistency-review-non-lean.md](docs/audit/2026-03-19-task-consistency-review-non-lean.md). Links task-plan drift to prior spec-audit findings to prepare for Rust code review.
- Specification consistency audit report for SPEC-001 through SPEC-018 in [docs/audit/2026-03-19-spec-001-018-consistency-review.md](docs/audit/2026-03-19-spec-001-018-consistency-review.md). Captures cross-spec inconsistencies and aligned areas without modifying the specs.

### Changed (continued 2)

- Clarified TASK-186 monitor-contract wording so exposed workflow obligations use `workflow_obligation_ref`, `MonitorLink` is shareable by default and distinct from control transfer, and [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) now records TASK-186 as a monitoring gate instead of renumbering the downstream convergence phases.
- Tightened TASK-177 core-contract wording so SPEC-001 scopes the runtime form set precisely, SPEC-002 treats optional binding and implicit `done` as surface sugar, and SPEC-004 gives explicit expression-level semantics for `Constructor` and `Match`. The core-language contract now separates canonical truth from surface convenience without widening runtime meaning to unrelated type-level contracts.
- `SPEC-001`, `SPEC-002`, and `SPEC-004` now separate canonical core truth from surface sugar and implementation convenience. The canonical IR contract is explicitly backend-neutral, so future interpreter and JIT implementations must preserve the same meaning rather than discover it locally.
- Reordered the convergence roadmap in [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) so a new spec-hardening gate now precedes Rust alignment phases. Parser/lowering, type/runtime, REPL/CLI, ADT, and final convergence work remain planned, but only after the language definition is tightened for mechanical Rust and Lean implementation.
- Tightened the workflow-declaration grammar in SPEC-002 so `observes` names `behaviour_ref` rather than a generic capability list. The grammar now preserves the existing semantic split between read-only behaviour inputs and separately declared write authority.
- Clarified workflow input declarations, `receive` scheduling terminology, and workflow communication/link wording across SPEC-002, SPEC-013, SPEC-014, SPEC-017, SPEC-018, and SPEC-020. The docs now distinguish `observes` from `receives`, reserve `policy` for authorization semantics, use `source scheduling modifier` for `receive` source selection, and define control-link transfer as consume-on-success.
- Canonicalized the ADT contract across SPEC-003, SPEC-004, SPEC-013, SPEC-014, and SPEC-020 (TASK-160). ADT declarations now use one `TypeDef`/`TypeExpr` source model, runtime variants store only constructor names plus fields, pattern and exhaustiveness rules share that same enum model, and the required Option/Result helper surface is explicitly narrowed.
- Canonicalized the REPL and CLI contract across SPEC-005, SPEC-011, and SPEC-016 (TASK-159). `ash repl` is now the sole normative REPL entrypoint, the REPL command set is limited to `:help`, `:quit`, `:type`, `:ast`, and `:clear`, and REPL display output is explicitly separated from workflow output capabilities.
- Canonicalized the stream and runtime-verification contract across SPEC-004, SPEC-013, SPEC-014, SPEC-017, and SPEC-018 (TASK-158). `receive` modes, control-arm behavior, declaration requirements, runtime-context responsibilities, and verification outcomes now share one end-to-end contract.
- Canonicalized the policy contract across SPEC-003, SPEC-004, SPEC-006, SPEC-007, SPEC-008, SPEC-017, and SPEC-018 (TASK-157). Policies now have one continuous story from named declaration and combinator expression through lowered core policy representation, type-checking constraints, and runtime `PolicyDecision` outcomes.
- Expanded [docs/plan/PLAN-INDEX.md](docs/plan/PLAN-INDEX.md) with logical post-Phase-20 convergence phases. The remaining convergence work is now split into docs-only handoff phases, implementation-alignment phases, and a final audit phase rather than living only inside the convergence plan document.

### Fixed (continued 1)

- ADT constructor typing, exhaustiveness, and runtime pattern tests now follow one constructor-shaped contract (TASK-174). `ash-typeck` now resolves variant patterns from canonical enum metadata in `TypeDef.body` instead of synthetic `__variant` record tags, exhaustiveness witnesses preserve required constructor field shape, and the focused ADT contract tests lock in constructor-shaped behavior end to end.
- Runtime boundary visibility now flows through explicit runtime-owned state rather than a temporary process-global fallback (TASK-206). `ash-interp` now exposes a `RuntimeState` carrier and stateful execution entrypoints, `ash-engine` now owns persistent runtime state across related executions, and focused engine/interpreter tests cover cross-execution control authority plus explicit rejection classes for missing capabilities and missing stream context.
- Runtime `Act` execution and control-link lifecycle handling now follow the hardened runtime contract (TASK-205). `ash-interp` now dispatches canonical `Act` workflows through registered operational capability providers, registers spawned control links for supervision, enforces reusable `pause` / `resume` / `check_health` behavior while an instance is live, invalidates future control operations after `kill`, and adds focused interpreter coverage for both the happy path and rejection path.
- Capability-level runtime policy outcomes now align across verification and interpreter execution (TASK-171). `ash-typeck` now treats approval and transform as distinct verification-time outcomes, `ash-interp` now applies capability-policy deny/approval/transform behavior consistently for `observe`, `set`, and `send`, and focused runtime policy tests cover the canonical contract.
- Hardened `ash-repl` error-formatting tests against ANSI-colored output so `cargo test --all` no longer fails nondeterministically in `src/lib.rs` unit tests. The REPL test suite now compares decolorized formatter output while preserving the colored runtime display path.
- End-to-end canonical `receive` execution now runs through the interpreter runtime (TASK-170). `ash-interp` now threads a shared mailbox through stream-aware recursive execution, executes lowered core `Workflow::Receive` forms directly, supports the implicit control mailbox, fails fast on missing runtime stream providers, and adds parsed-form integration tests for non-blocking, blocking, timed, and control receive behavior plus the runtime-verification input split regression.
- Separated aggregate runtime-verification inputs for workflow capability declarations versus obligation-backed requirements (TASK-209). `ash-typeck` now requires explicit `AggregateVerificationInputs`, stops deriving obligation requirements from `WorkflowCapabilities`, checks operation-class capability requirements separately from runtime roles and named obligations, and adds contract tests for the split.
- Type-checking and runtime-verification convergence for TASK-168 and TASK-169. `ash-typeck` now rejects policy-less `decide`, rejects policy targets at `check`, enforces declared stream bindings for canonical `receive`, restores aggregate required-capability enforcement in runtime verification, and carries the canonical runtime verification context fields needed for the hardened contracts.
- Parser/lowering convergence for TASK-164 through TASK-167. `receive` now routes through the main workflow parser, canonical `decide { ... } under <policy>` and obligation-only `check` forms are enforced, surface `receive` lowers into the canonical core `Workflow::Receive`, and lexer recovery no longer skips valid tokens after an unexpected character.
- Restored `ash-cli` compatibility with boxed `Value::List` and `Value::Record` constructors, and moved binary command tests into an integration harness so `cargo test -p ash-cli` passes again on the workflow-contracts branch.

### Changed

- Canonicalized the spec contracts for `check`, `decide`, and `receive` across SPEC-001, SPEC-002, SPEC-003, SPEC-004, SPEC-017, and SPEC-018 (TASK-156). `check` is now obligation-only, `decide` always names an explicit policy, and `receive` is documented as an epistemic mailbox-input form with one authoritative surface grammar.

### Added

- Formal proofs for semantic properties (Phase 19, TASK-149 through TASK-155):
  - `Ash/Proofs/Pattern.lean` - Pattern match determinism and totality proofs
  - `Ash/Proofs/Pure.lean` - Constructor purity proof (effect system)
  - `Ash/Proofs/Determinism.lean` - Expression evaluation determinism proof
  - `Ash/Proofs/Progress.lean` - Progress theorem (well-typed programs don't get stuck)
  - `Ash/Proofs/Preservation.lean` - Preservation theorem (types preserved during evaluation)
  - `Ash/Proofs/TypeSafety.lean` - Type safety corollary combining progress and preservation
  - `Ash/Types/Basic.lean` - Core type system definitions (`Ty` inductive)
  - `Ash/Types/WellTyped.lean` - Well-typed relation for expressions
  - Helper lemmas: `merge_envs_assoc`, `env_lookup_bind_eq`, `join_epistemic_left`, etc.
  - **Note**: Some theorems use `sorry` due to Lean 4 partial function limitations
- Effect tracking for receive capability (TASK-108). Complete effect tracking for all capabilities:
  - Added `Workflow::Receive` variant to surface AST for pattern matching on incoming messages
  - Added `ReceiveMode` enum (NonBlocking, Blocking with optional timeout)
  - Added `StreamPattern` enum (Wildcard, Literal, Binding) for receive arm patterns
  - Added `ReceiveArm` struct (pattern, guard, body, span)
  - Implemented effect computation: receive is `Epistemic` (read-only consumption) per SPEC-017
  - Effect properly joins with all arm body effects: `arms.iter().map(|arm| arm.body.effect()).fold(Epistemic, join)`
  - Added 7 property tests for receive effect tracking (empty, blocking, epistemic body, operational body, multiple arms, control receive)
  - Updated desugar passes (sequencing, optional bindings, nested blocks) to handle Receive
  - Updated lowering with placeholder for future core IR support
  - Verified compliance with SPEC-017 Section 2.1: receive → Epistemic effect
- Option and Result standard library (TASK-136). Core standard library modules:
  - `std/src/option.ash` - Option<T> type with Some/None variants
  - `std/src/result.ash` - Result<T, E> type with Ok/Err variants
  - Helper functions: is_some, is_none, is_ok, is_err, unwrap, unwrap_or, unwrap_err
  - Transformation functions: map, map_err, and_then, and, or, ok_or, ok, err
  - `std/src/prelude.ash` - Auto-imported types and functions
  - `std/src/lib.ash` - Main library exports
  - `std/README.md` - Standard library documentation
  - Integration tests verifying stdlib files parse correctly
- Spawn returns Instance with Option<ControlLink> (TASK-134). Updated spawn expression to return a composite type that can be split into InstanceAddr and Option<ControlLink>:
  - Added `Instance`, `InstanceAddr`, and `ControlLink` types to `ash-core` value module
  - Added `Value::Instance`, `Value::InstanceAddr`, `Value::ControlLink` variants for runtime representation
  - Added `Expr::Spawn { workflow_type, init }` expression for spawning workflows
  - Added `Expr::Split` expression to decompose Instance into (InstanceAddr, ControlLink)
  - Added `Workflow::Spawn` and `Workflow::Split` workflow variants
  - Implemented evaluation logic in `ash-interp` for spawn (creates Instance with unique ID) and split (returns tuple)
  - Added visualization support for new workflow variants
  - Full test coverage for spawn/split evaluation and instance value display
- Affine control link transfer semantics (TASK-135). Runtime tracking for control link consumption:
  - `ControlLinkRegistry` for tracking link availability vs consumed state
  - `ControlLinkError` for invalid link usage (AlreadyConsumed, NotFound, InvalidInstance)
  - `acquire()` method for consuming links with exactly-once semantics
  - `verify_unused()` for checking link availability without consuming
  - `consume()` for explicit consumption, `is_consumed()` for state checking
  - Support for kill, pause, resume, check_health supervision operations
  - Workflow variants: Kill, Pause, Resume, CheckHealth for supervision
- Match and if-let expression evaluation (TASK-133). Interpreter support for match expressions:
  - `Expr::Match` evaluation with pattern matching and arm selection
  - `Expr::IfLet` evaluation as sugar for match
  - Integration with pattern matching engine for variable binding
  - Proper error handling for non-exhaustive matches
  - Full test coverage for all match forms
- Pattern matching engine (TASK-132). Core pattern matching implementation in `crates/ash-interp/src/pattern.rs`:
  - `Value::Variant` type added to `ash-core` for representing variant values
  - `Pattern::Variant` pattern matching with field extraction
  - Support for unit variants: `Pattern::Variant { name: "None", fields: None }`
  - Support for variants with fields: `Pattern::Variant { name: "Some", fields: Some([("value", var)]) }`
  - Nested variant pattern matching (variants containing tuples, records, etc.)
  - Full test coverage for variant matching including negative cases
- Constructor evaluation for ADTs (TASK-131). Interpreter support for evaluating constructor expressions like `Some { value: 42 }`:
  - `Value::Variant` type in `ash-core` with constructor name and field values
  - `Expr::Constructor` evaluation in `ash-interp/src/eval.rs`
  - Helper methods: `Value::variant()` and `Value::unit_variant()` for creating variants
  - Support for nested constructors, expressions in fields, and variable references
  - Full test coverage for Option, Result, and custom ADT constructors

### Fixed

- Dead code review: 5 `#[allow(dead_code)]` items audited, 2 duplicate `ws()` functions identified for removal
- Code review issues from Phase 17 (P0, P1, P2 priority):
  - **Critical (P0)**: Fixed `unwrap()` abuse in parsers (`parse_pattern.rs`, `parse_expr.rs`) using `is_some_and()`
  - **Critical (P0)**: Removed unnecessary `Box::new` + immediate dereference pattern in `lower.rs`
  - **High (P0)**: Added `#[must_use]` to all public constructors and pure functions in `exhaustiveness.rs`, `instantiate.rs`, `type_env.rs`
  - **High (P1)**: Boxed large `Value` enum variants (`List`, `Record`, `Variant`, `Instance`) to reduce memory footprint
  - **High (P1)**: Removed broken ternary expression parsing from `parse_expr.rs`
  - **Medium (P2)**: Added `HashMap::with_capacity()` hints where collection size is known
  - **Medium (P2)**: Optimized pattern matching to avoid temporary HashMap allocation
  - **Low (P2)**: Removed dead code/comments from parser files
  - **Low (P2)**: Fixed float literal lowering to truncate to Int instead of returning Null
- Type definition duplication between `ash-core` and `ash-typeck`. Unified `TypeDef` types by using AST types from `ash_core::ast` in `type_env.rs` with conversion functions.
- Inefficient TypeEnv creation in pattern checking. Added static `EMPTY_ENV` with `OnceLock` to avoid repeated allocations.
- Keyword lookup performance. Replaced O(n) `matches!` pattern with O(1) `HashSet` lookup using `OnceLock` for lazy initialization.
- Magic string for variant tag. Extracted `"__variant"` to `const VARIANT_TAG` constant.
- Visibility enum completeness. Added `Crate` variant to `Visibility` enum.
- Unsafe `unwrap()` usage in parser. Replaced with `is_some_and()` pattern.
- Error message formatting. Changed to lowercase per Rust conventions.

### Added (continued 2)

- Match and if-let expression evaluation (TASK-133). Pattern matching in the interpreter:
  - `eval_match()` function for evaluating `Expr::Match` with multiple arms
  - `eval_if_let()` function for evaluating `Expr::IfLet` expressions
  - Pattern matching using existing `match_pattern()` engine
  - Variable bindings scoped to match arm bodies via `Context::extend()`
  - `NonExhaustiveMatch` error when no arm matches
  - Support for all pattern types: literal, variable, wildcard, tuple, record, list
  - First matching arm wins semantics
  - If-let desugars to match with pattern/then/else branches
- Generic type instantiation (TASK-129). Type parameter substitution for ADTs:
  - `instantiate(def, args)` function for substituting type parameters with concrete types
  - `Substitution::from_pairs()` method for creating substitutions from type variable pairs
  - `InstantiateError::ArityMismatch` for wrong number of type arguments
  - Support for instantiating enums, structs, and type aliases
  - Recursive substitution in nested types (tuples, records, constructors)
  - Full test coverage for single and multi-parameter type definitions
- Type check patterns for match expressions (TASK-128). Pattern type checking in `crates/ash-typeck/src/check_pattern.rs`:
  - `check_pattern(env, pattern, expected)` function for checking patterns against expected types
  - `Bindings` type: `HashMap<String, Type>` for pattern variable bindings
  - Support for `Pattern::Wildcard` - matches any type with no bindings
  - Support for `Pattern::Variable` - binds variable to expected type
  - Support for `Pattern::Literal` - checks literal type compatibility
  - Support for `Pattern::Variant` - checks variant patterns against sum types
  - Support for `Pattern::Tuple` - checks element count and types
  - Support for `Pattern::Record` - checks field names and types
  - Support for `Pattern::List` - checks element patterns and rest bindings
  - New error types: `PatternMismatch`, `UnknownVariant`, `PatternArityMismatch`, `InvalidPattern`
  - `TypeEnv` for managing type definitions and variable scopes during pattern checking
  - Full test coverage for all pattern types including nested patterns
- Type check constructors for ADTs (TASK-127). Type checking for constructor expressions like `Some { value: 42 }`:
  - `TypeEnv` struct to track type definitions and constructor mappings
  - `register_type(def: TypeDef)` to add type definitions
  - `lookup_constructor(name)` to find constructor's type and variant index
  - `lookup_type(name)` to retrieve type definitions
  - `add_builtin_types()` to register Option and Result types
  - `check_expr` function with `Expr::Constructor` case for expression type checking
  - Error types: `UnknownConstructor`, `MissingField`, `UnknownField`
  - Full test coverage for Option and Result constructors
- Parse type definitions (TASK-124). Parser for ADT type definitions in `ash-parser`:
  - `parse_type_def` module with `TypeDef`, `TypeBody`, `VariantDef`, `Visibility`, and `TypeExpr` types
  - Support for enums: `type Status = Pending | Processing | Completed;`
  - Support for struct types: `type Point = { x: Int, y: Int };`
  - Support for type aliases: `type Name = String;`
  - Support for generics: `type Option<T> = Some { value: T } | None;`
  - Support for visibility: `pub type Result<T, E> = Ok { value: T } | Err { error: E };`
  - Full test coverage for all type definition forms
- AST Extensions for Algebraic Data Types (TASK-120). Foundation for Phase 17 ADT implementation:
  - `Pattern::Variant` for enum variant pattern matching
  - `Expr::Constructor` for ADT value construction
  - `Expr::Match` for pattern matching expressions
  - `Expr::IfLet` for if-let syntactic sugar
  - `MatchArm` struct representing match arms
  - `TypeDef`, `TypeBody`, `VariantDef` for type definitions
  - `Visibility` enum for visibility modifiers (pub, crate, private)
  - `TypeExpr` for surface syntax type expressions
  - `Type::Instance`, `Type::InstanceAddr`, `Type::ControlLink` for spawn/control link support
- Stream iteration over registered streams. Added `StreamRegistry::iter()` method to iterate over all registered providers, `StreamContext::iter_providers()` to iterate over typed providers, and `StreamContext::try_recv_any()` to receive from any available stream (non-blocking). Updated `wait_for_message()` in `execute_stream.rs` to poll all registered streams using `try_recv_any()` instead of busy-waiting.

### Fixed

- Infinite recursion bug in `TypedSendableProvider::send()` and `BidirectionalStreamProvider::send()` methods. Both were calling themselves instead of delegating to `inner.send()`. Added proper write_schema validation and delegation to inner provider.

### Changed (continued 3)

- Refactored parser utilities to eliminate code duplication between `parse_set.rs` and `parse_send.rs`. Created new `parse_utils.rs` module with shared helper functions: `parse_capability_ref()`, `keyword()`, `literal_str()`, and `skip_whitespace_and_comments()`.

### Added (continued 3)

- Set statement execution for output behaviours (TASK-105). New `execute_set` module in `ash-interp` with `execute_set(capability, channel, value, behaviour_ctx)` async function for setting values on writable channels. Integrates with `BehaviourContext` to lookup settable providers, validates values before setting, and returns `ExecError::CapabilityNotAvailable` or `ExecError::ValidationFailed` on errors. Added `Workflow::Set` variant to AST with `capability`, `channel`, and `value` fields. Extended `execute_workflow` with new `execute_workflow_with_behaviour` function that accepts `BehaviourContext` for set statement support.
- Parse send statement for output streams (TASK-104). New `parse_send` module in `ash-parser` with `SendExpr` struct for parsing `send capability:channel expr` syntax. Similar to `parse_set` but without the `=` sign. Supports variables, string literals, and function calls for structured values.
- Parse set statement for output behaviours (TASK-103). New `parse_set` module in `ash-parser` with `SetExpr` struct for parsing `set capability:channel = expr` syntax. Supports simple values, function calls for structured values, and expressions.
- Sendable Stream Provider Trait (TASK-102). Output capability support for writable streams:
  - `SendableStreamProvider` trait extending `StreamProvider` with `send(&self, value: Value)` async method
  - `would_block(&self) -> bool` for backpressure detection (default: false)
  - `flush(&self)` async for buffered sends (default: no-op)
  - `TypedSendableProvider` wrapper with `write_schema` validation before sending values
  - `MockSendableProvider` for testing with `sent_values()` and `sent_count()` inspection
  - `SendableRegistry` for managing sendable providers by capability/channel
  - `StreamContext` extension with `register_sendable()`, `get_sendable()`, and `send()` methods
- Settable Behaviour Provider Trait (TASK-101). Output capability support for writable channels:
  - `SettableBehaviourProvider` trait extending `BehaviourProvider` with `set(&self, value: Value)` async method and optional `validate(&self, value: &Value)` for pre-checks
  - `TypedSettableProvider` wrapper with `write_schema` validation before setting values
  - `MockSettableProvider` for testing with configurable validators
  - `SettableRegistry` for managing settable providers by capability/channel
  - `BehaviourContext` extension with `register_settable()`, `get_settable()`, and `set()` methods
  - `ValidationError` enum with variants for invalid values, out of range, and format errors
  - `ExecError::ValidationFailed` variant for validation failure reporting
- Bidirectional Provider Wrappers (TASK-107). Combine input/output capabilities for unified providers:
  - `BidirectionalBehaviour` trait combining `sample()` and `set()` operations for internal implementations
  - `BidirectionalBehaviourProvider` wrapper implementing both `BehaviourProvider` and `SettableBehaviourProvider` with separate `read_schema` and `write_schema` validation
  - `MockBidirectionalProvider` for testing with read/write operation tracking via `read_count()` and `write_count()`
  - `BidirectionalStream` trait combining `recv()`/`try_recv()` and `send()` operations for internal implementations
  - `BidirectionalStreamProvider` wrapper implementing both `StreamProvider` and `SendableStreamProvider` with separate read/write schema validation
  - `MockBidirectionalStream` for testing with `push()` for receive queue and `sent_values()`/`sent_count()` for sent values inspection
- Phase 16: Runtime Verification (TASK-114 to TASK-119). Comprehensive runtime verification framework:
  - Capability availability verifier (TASK-114). New `CapabilityVerifier` checks all required capabilities are available with correct modes (observable, settable, sendable, receivable).
  - Obligation satisfaction checker (TASK-115). New `RuntimeObligationChecker` verifies role requirements and obligation presence at runtime.
  - Effect compatibility checker (TASK-116). New `EffectChecker` ensures workflow effect level is within runtime bounds.
  - Static policy validator (TASK-117). New `StaticPolicyValidator` detects always-denied operations and approval requirements pre-execution.
  - Per-operation runtime verifier (TASK-118). New `OperationVerifier` with async `verify()` for checking capability availability, mode support, policy evaluation, and rate limiting.
  - Verification aggregator (TASK-119). New `VerificationAggregator` combines all verifiers into unified `VerificationResult` with `can_execute()` determination.
- Phase 15: Capability Integration (TASK-108 to TASK-113). Full integration of capabilities with obligations, policies, provenance, and type safety:
  - Effect tracking for all capability operations (TASK-108). Added `Workflow::effect()` method that computes total effect by joining operation effects (Observe/Receive=Epistemic, Set/Send=Operational).
  - Obligation checking with capabilities (TASK-109). New `ObligationChecker` verifies workflows have required input/output capabilities and sufficient effect levels.
  - Policy evaluation for input/output (TASK-110). New `CapabilityPolicyEvaluator` with support for Permit, Deny, RequireApproval, and Transform decisions.
  - Provenance tracking for all capabilities (TASK-111). New `CapabilityProvenanceTracker` records all capability operations with event types, values, and policy decisions.
  - Capability declaration verification (TASK-112). New `CapabilityChecker` framework for verifying workflows use declared capabilities.
  - Read/write type checking (TASK-113). New `CapabilitySchemaRegistry` validates input/output values against provider schemas with separate read/write types.
- Phase 14: Typed Providers (TASK-096 to TASK-100). Runtime type safety for Rust/Ash provider boundary:
  - `TypedBehaviourProvider` and `TypedStreamProvider` wrapper structs carrying type schemas (TASK-096)
  - Schema validation logic with `Type::matches()` and `Type::validate()` methods (TASK-097)
  - Typed registry integration - `BehaviourRegistry` and `StreamRegistry` now store typed providers with schema lookup via `get_schema()` (TASK-098)
  - Runtime validation in providers - sample/recv operations validate values against schemas (TASK-099)
  - Enhanced type error reporting with `ExecError::TypeMismatch` and path tracking (TASK-100)
- Shared capability types module (ash-core). New `capability.rs` consolidates `Direction`, `RoleName`, `RequiredCapabilities`, and `WorkflowCapabilities` to eliminate duplication across crates.
- Phase 13: Streams and Behaviours (TASK-088 to TASK-095). Complete stream processing and behaviour sampling implementation:
  - Stream AST types: `StreamRef`, `Receive`, `ReceiveMode`, `Mailbox` with overflow strategies (TASK-088)
  - Stream provider trait with `StreamRegistry` and `StreamContext` for async stream operations (TASK-089)
  - Parse receive construct with guards, timeouts, and control streams (TASK-090)
  - Mailbox implementation with size limits and overflow strategies (DropOldest, DropNewest, Error) (TASK-091)
  - Stream execution with pattern matching, guard evaluation, blocking/non-blocking modes (TASK-092)
  - Behaviour provider trait with `BehaviourRegistry` and `BehaviourContext` for sampling (TASK-093)
  - Parse observe construct with constraints (TASK-094)
  - Observe execution with sampling and pattern binding (TASK-095) New `execute_observe` module in `ash-interp` provides `execute_observe()` and `execute_changed()` functions. `execute_observe()` samples behaviour providers with constraints, matches patterns against sampled values, and binds variables. `execute_changed()` detects value changes since last sample. Includes 6 comprehensive async tests and proper error handling for missing providers and pattern match failures.
- Stream execution with pattern matching and guards (TASK-092). New `execute_stream` module in `ash-interp` provides `execute_receive` function supporting non-blocking/blocking/timeout modes, pattern matching with destructuring, guard clause evaluation, and control stream handling. Includes 10 comprehensive async tests.
- Interactive REPL (Phase 12, TASK-077 to TASK-083). New `ash-repl` crate with rustyline integration provides expression evaluation, multi-line input detection, commands (:help, :quit, :type, :ast, :clear), tab completion for keywords, persistent history, and syntax error highlighting with helpful suggestions.
- Embedding API for ash-engine crate (Phase 11, TASK-071 to TASK-076). Unified Engine type with Parse→Check→Execute lifecycle, builder pattern (EngineBuilder), thread-safe workflow storage, and capability provider traits. CLI integration complete with 160 tests passing.

### Changed (continued 4)

- Updated dependencies to latest versions: winnow 0.5.40 → 0.6.26, pulldown-cmark 0.9.6 → 0.13.1, thiserror 1.0.69 → 2.0.18, colored 2.1 → 3.1.1. Fixed winnow API migration (PResult → ModalResult, Located → LocatingSlice) and pulldown-cmark breaking changes (TagEnd::CodeBlock, CodeBlockKind).
- Fixed all clippy warnings (66+ style and correctness warnings). Removed redundant pattern matching, fixed `#[must_use]` attributes, added `#[allow]` annotations for intentional patterns.
- Fixed test failures: updated forall/exists tests to use non-keyword identifiers; removed method_chain test (feature not in spec); fixed error_recovery test assertion.
- **Breaking**: Z3/SMT is now a mandatory dependency (removed `smt` feature flag). Policy conflict detection is always enabled for security-critical workflows. System must have Z3 C library installed.

### Added (continued 4)

- List literal parsing for expressions: `[1, 2, 3]` or `["a", "b"]` syntax. Updated SPEC-002 to define list_literal production. Added Literal::List variant to surface AST.

### Added (continued 5)

- Initial project structure with workspace and 9 crates (ash-core, ash-macros, ash-parser, ash-typeck, ash-interp, ash-provenance, ash-cli, ash-lint, ash-doc-tests)
- Effect lattice implementation with 4 levels: Epistemic, Deliberative, Evaluative, Operational (TASK-001)
- Comprehensive property tests for Effect lattice: associativity, commutativity, idempotence, absorption, identity (18 property tests)
- Value system with 9 variants: Int, String, Bool, Null, Time, Ref, List, Record, Cap (TASK-002)
- Value serialization/deserialization with JSON roundtrip property tests (17 property tests)
- Core AST definitions for workflow language (SPEC-001)
- AST visualization module generating Graphviz DOT output
- Comprehensive development tooling: git hooks, sccache, insta, proptest
- CI/CD plan with 6 workflow types and initial ci-fast.yml implementation
- Documentation: 5 specification documents, architecture document, CLI specification
- Custom lint tool (ash-lint) for Ash-specific rules
- Doc-test extractor for testing code examples in specifications
- Fuzz testing infrastructure with cargo-fuzz (ash-fuzz crate)
- Benchmark suite with Criterion (ash-bench crate)
- Procedural macros for Effectful and Provenance derive
- Serde Serialize/Deserialize support for all AST types: Workflow, Pattern, Expr, Guard, etc. (TASK-003)
- List pattern variant for prefix matching with optional rest binding: `List(Vec<Pattern>, Option<Name>)` (TASK-003)
- Pattern helper methods: `bindings()` to collect variable names, `is_refutable()` to check match exhaustiveness (TASK-003)
- Comprehensive AST tests: workflow construction, pattern bindings, serde roundtrip (TASK-003)
- Provenance tracking types: WorkflowId, Provenance, TraceEvent, Decision with fork lineage (TASK-004)
- Provenance tests: lineage accumulation, uniqueness, serde roundtrip (TASK-004)
- Pattern matching system with 6 variants: Variable, Tuple, Record, List, Wildcard, Literal (TASK-005)
- Pattern helper methods: bindings() for collecting variables, is_refutable() for exhaustiveness (TASK-005)
- Property testing strategies: arb_effect, arb_value, arb_pattern, arb_name, arb_expr (TASK-006)
- Proptest helpers tests: binding uniqueness, value roundtrip, name validation (TASK-006)
- Test helpers module: WorkflowBuilder, test_capability, var, lit, var_expr utilities (TASK-007)
- 13 test helper tests for builders and utilities (TASK-007)
- Token definitions with 50+ variants: keywords, literals, operators, delimiters (TASK-008)
- Span tracking for source locations with line/column/byte offset (TASK-008)
- LexError types with thiserror for unexpected chars, unterminated strings, invalid numbers (TASK-008)
- Lexer implementation with streaming tokenization, comments, error recovery (TASK-009)
- 16 lexer tests for keywords, identifiers, literals, operators, spans, recovery (TASK-009)
- 23 lexer property tests: identifiers, literals, spans, error recovery, stress tests (TASK-010)
- Workflow parser with 18 tests: observe, act, let, if, for, par, etc. (TASK-013)
- Expression parser with 22 tests: precedence climbing, literals, binary ops (TASK-014)
- Error recovery with 12 tests: synchronization, recovery strategies (TASK-015)
- Surface to Core lowering with 17 tests: workflow, expr, pattern lowering (TASK-016)
- Desugaring with 17 tests: sequencing, optional bindings, nested blocks (TASK-017)
- Lexer property tests: 18 proptest-based tests for identifiers, literals, spans, error recovery, and stress testing (TASK-010)
- Surface AST types for parser: Program, Definition, Workflow, Expr, Pattern, and supporting types with full span tracking (TASK-011)
- 49 surface AST tests: construction tests for all major types, span extraction tests, and variant coverage (TASK-011)
- Parser core using winnow: ParseInput with Stream impl, ParseError with span tracking, basic combinators (TASK-012)
- 25 parser core tests: ParseInput Stream operations, ParseError formatting, whitespace/alphanumeric/keyword combinators (TASK-012)
- CLI implementation with 5 commands: check, run, trace, repl, dot (TASK-053 to TASK-057)
- check command with --all, --strict, --format flags for type checking workflows
- run command with --input, --output, --trace flags for workflow execution
- trace command with provenance capture and JSON/NDJSON/CSV export formats
- repl command with rustyline integration, :help, :type, :bindings commands
- dot command for Graphviz DOT output generation
- 23 CLI tests for argument parsing, command execution, and help output
- Example workflows: 12 examples across 4 categories (basics, control-flow, policies, real-world) (TASK-047)
- Examples README with overview, quick start, and learning path
- Basics examples: hello-world, variables, expressions, observe pattern
- Control flow examples: conditionals, foreach, parallel, sequential
- Policy examples: role-based and time-based access control
- Real-world examples: customer support and code review workflows
- Comprehensive tutorial covering installation through real-world examples (TASK-048)
- API documentation for all crates: ash-core, ash-parser, ash-typeck, ash-interp, ash-provenance, ash-cli (TASK-049)
- Core benchmarks: effect operations, value operations, pattern matching (TASK-050)
- Parser benchmarks: simple, complex, and nested workflow parsing
- Interpreter benchmarks: workflow construction, expression evaluation, traversal
- Serialization benchmarks: JSON roundtrip for workflows and values
- Optimization documentation: performance characteristics and tuning guide (TASK-051)
- Parser fuzzing target for validating input handling (TASK-052)
- Type checker fuzzing target for crash detection
- Module resolution algorithm (TASK-069). Implemented `ModuleResolver` with file system abstraction trait for testability, supporting Rust-style module resolution (`mod foo;` → `foo.ash` or `foo/mod.ash`). Includes circular dependency detection, proper error handling with `ResolveError`, and `MockFs` for testing. 19 comprehensive tests covering single files, nested modules, directory modules, and circular dependencies.
- Policy combinators implementation with 12 AST variants: Var, And, Or, Not, Implies, Sequential, Concurrent, ForAll, Exists, MethodCall, Call (TASK-062)
- Policy expression parser with support for infix operators (&, |, !, >>), method chaining (.and(), .or(), .retry()), and quantifiers (forall, exists) (TASK-062)
- Policy type checker with 21 tests: type inference, validation, method signatures, context bindings (TASK-062)
- Policy normalization passes: flatten nested and/or, eliminate double negation, constant folding preparation (TASK-062)
- 12 surface AST tests for PolicyExpr variants: construction, span extraction, variant coverage (TASK-062)
- Visibility checking for type checker (TASK-070). Implemented `VisibilityChecker` with `check_access` method for validating item accessibility across module boundaries. Supports all visibility variants: `pub`, `pub(crate)`, `pub(super)`, `pub(self)`, and `pub(in path)`. Includes `VisibilityError` enum with `PrivateItem` and `MissingContext` error variants. 17 comprehensive tests covering all visibility scenarios.
- ash-engine crate with unified Engine type for embedding (TASK-071). Created new crate with `Engine` struct providing unified interface for Parse → Check → Execute workflow. Engine implements `Send + Sync` for thread safety. Builder pattern via `EngineBuilder` with fluent API for capability configuration. 39 tests covering engine creation, configuration, and error handling.
- Engine::parse and Engine::parse_file methods (TASK-072). Implemented source string and file path parsing with automatic lowering from surface AST to core IR. 29 comprehensive tests including valid workflows, invalid syntax, file I/O, and property tests for error preservation.
- Engine::check method for type checking (TASK-073). Integrated with ash_typeck to validate workflows. Creates wrapper type carrying surface workflow for type checker compatibility. Added `ret` keyword support across parser, lexer, surface AST, lowering, and type checking. 28 tests covering type checking scenarios.
- Engine::execute, run, and run_file methods (TASK-074). Async execution methods providing full pipeline (parse → check → execute) and individual execution. Integrated with ash_interp for workflow interpretation. 32 tests including async behavior, concurrent execution, and error handling.
- Standard capability providers (TASK-075). Implemented `StdioProvider` (print, println, read_line) and `FsProvider` (read_file, write_file, exists) with `CapabilityProvider` trait. Builder methods `with_stdio_capabilities()` and `with_fs_capabilities()` on EngineBuilder. 28 tests covering provider behavior and trait implementations.
- CLI integration with ash-engine (TASK-076). Updated ash-cli to use Engine API instead of direct crate dependencies. `ash run` command now uses Engine::run_file with stdio/fs capabilities. `ash check` command uses Engine::parse + Engine::check. All 23 CLI tests pass with new implementation.

### Changed (reserved)

### Deprecated

### Removed

- Removed `par` workflow form from parser, lexer, and lowering (TASK-448). The `par { ... }` parallel workflow syntax is no longer supported. Removed from token.rs, lexer.rs, parse_workflow.rs, desugar.rs, lower.rs, error_recovery.rs, lexer_props.rs, and ash-engine/src/lib.rs.

### Fixed

### Security
