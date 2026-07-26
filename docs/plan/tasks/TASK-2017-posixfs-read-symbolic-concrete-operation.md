# TASK-2017: `PosixFs::read` Symbolic Concrete Operation

**Status:** Complete — normal local declaration resolution now carries nominal `PosixFs::read`
through exact non-granting-row checking, explicit admitted provider binding, deterministic
controlled dispatch, and private checked-CPS String-atom inspection. Imports, generics, handlers,
production Core/CPS execution, and real `FsProvider` host reads remain excluded.
**Phase:** Implementation follow-up from [TASK-2010](TASK-2010-static-impl-operation-source-call.md),
[TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md),
[TASK-2012](TASK-2012-declared-operation-provider-binding.md),
[TASK-2015](TASK-2015-evaluated-local-arguments-symbolic-operation-calls.md), and
[TASK-2016](TASK-2016-local-nominal-newtype-checking.md)

## Description

Realize one normal, declaration-resolved symbolic concrete operation end to end:
`PosixFs::read(path)`, where `path` is either a checked string literal or an already evaluated
checked local `String`.  The source call must derive its identity, `String -> String` signature,
non-granting row item, admission key, provider-operation binding, direct-runtime dispatch, and
private checked CPS `Raise` from the same registered `Fs`/`PosixFs` declarations.  It is not a
legacy string `invoke` compatibility route, a raw qualified-name registry, or host filesystem
access by default.

TASK-2016 supplies the necessary local nominal `PosixFs` identity.  This task does not reopen
the semantics of `ImplType::operation(args)`: the target contract already fixes that a successful
call contributes `PosixFs::read`, requires explicit discharge, and lowers as an operation `Raise`.

## Authoritative References

- [SPEC-097b §3.3, §7.2, and §8.1](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): concrete
  impl-qualified operation identity, `String` operation signature example, inferred requirement
  row, and no row-as-authority interpretation.
- [SPEC-098c §6–7](../../spec/SPEC-098c-SURFACE-TO-CORE-LOWERING.md): lowering retains canonical
  `ImplType::op` identity and never synthesizes provider authority from a row.
- [SPEC-099b §5](../../spec/SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md): provider-frame lookup and
  operation dispatch use impl-qualified identity; an unhandled operation is a structured failure.
- [ASH-CPS-CALCULUS](../../spec/ASH-CPS-CALCULUS.md): `SEM-EFFECT-LOOKUP-001` and
  `SEM-EFFECT-RAISE-001` are the operation lookup and raising rule identities.
- [TASK-2011](TASK-2011-declared-concrete-impl-operation-source-calls.md): declaration-backed
  resolution rather than source/name-string inference.
- [TASK-2012](TASK-2012-declared-operation-provider-binding.md): exact typed provider-operation
  binding and reject-before-dispatch admission.
- [TASK-2015](TASK-2015-evaluated-local-arguments-symbolic-operation-calls.md): evaluated-local
  transport and private `Raise` inspection boundary.
- [TASK-2016](TASK-2016-local-nominal-newtype-checking.md): local nominal impl-type identity.

## Scope

### In scope

- A local, non-generic declaration fixture that proves `PosixFs` is the concrete implementation
  of `Fs` and that `read` has the declared `String -> String` signature.
- `PosixFs::read("fixture-path")` and `let path = "fixture-path"; PosixFs::read(path)` on the
  ordinary parse/check/entry path.
- Resolution to exactly `DeclaredConcreteOperation { impl_type: PosixFs, operation: read, ... }`,
  an exact non-granting `PosixFs::read` row, and an explicit matching binding/admission route.
- `FsProvider` (or its narrowly extracted metadata seam) declaring an exact bindable provider
  operation compatible with the canonical identity, while preserving the existing provider
  authoring validation.
- Deterministic direct-runtime evidence through a test-only or controlled injected provider that
  maps a fixed string path to fixed string content.  The positive test must not read a real host
  file and must show one dispatch with the exact `Value::String` argument.
- Private checked CPS inspection that produces
  `Raise(PosixFs::read, [Atom::String(path)], {PosixFs::read})` with declared `String -> String`
  types.  The string atom conversion must be value-preserving and must fail closed for unsupported
  values.
- Negative evidence for no binding/no admitted provider, wrong argument type, provider-operation
  metadata mismatch, and retained direct-source `invoke` rejection.

### Explicit exclusions

- Arbitrary real host filesystem reads, sandbox policy design, path normalization, TOCTOU,
  directory traversal, streaming, writes, or errors-as-language-values.
- Imports/re-exports/cross-module coherence, generic or interface-qualified operation calls,
  specialization, binding-name calls, multi-provider selection, and general expression evaluation.
- Handler runtime/frame installation, residual-row subtraction, production Core/CPS execution,
  or direct-runtime/CPS parity claims.
- Any raw provider/action string fallback, operation-tail matching, row-text matching, or
  restoration of deleted source `invoke` syntax.

## Requirements and Invariants

1. **Declaration identity:** normal checking must resolve the source spelling only through
   registered local declaration facts, yielding canonical `PosixFs::read`; neither the lexical
   variable name nor provider metadata may choose operation identity.
2. **Signature and nominal identity:** only one `String` argument is accepted and the result is
   `String`; `PosixFs` remains the TASK-2016 nominal identity rather than a transparent alias or
   source-text convention.
3. **Authority neutrality:** the checked row is exactly one deduplicated `PosixFs::read`
   requirement.  It grants neither an `FsProvider` nor access to a host filesystem.  Missing or
   mismatched binding/admission rejects before provider dispatch.
4. **Exact binding:** the selected provider is bound by validated metadata and the complete
   declared operation identity/signature.  A nearby `fs.read`, `read`, or unrelated provider
   operation cannot satisfy the binding by prefix, suffix, or spelling coincidence.
5. **Deterministic dispatch:** the positive direct-runtime test provider observes exactly one
   `Value::String("fixture-path")` vector and returns only its fixed fixture content.  Production
   defaults must not acquire an implicit real-file fallback from this task.
6. **CPS shape:** private inspection carries the same declared identity, signature, local row, and
   `Atom::String` argument as direct execution.  It remains inspection only under TASK-2004.
7. **No regression:** direct source `invoke` remains rejected and existing `TestClock::sleep`
   descriptor/declaration/binding behavior continues unchanged.

## TDD Sequence

1. **Freeze the seams.** Locate `DeclaredConcreteOperation` resolution in `ash-typeck`, normal
   `Engine::check` row transport, `DeclaredOperationProviderBinding` validation/dispatch in
   `crates/ash-engine/src/lib.rs`, `FsProvider` metadata, and the checked-CPS atom/lowering path.
   Record the concrete declaration fixture and exact provider metadata name before modifying code.
2. **RED: declaration-backed literal resolution.** Add
   `crates/ash-engine/tests/task_2017_posixfs_read_symbolic_operation.rs` using an ordinary local
   `Fs`/`PosixFs` declaration fixture and `PosixFs::read("fixture-path")`.  Assert normal
   parse/check yields declared `String -> String` identity and exactly one non-granting
   `PosixFs::read` row.  Add wrong-arity/wrong-type and unknown-operation controls.
3. **GREEN: resolve only the declared operation.** Extend the existing declaration resolver or
   registration bridge just enough to recognize this local concrete `Fs` operation.  Do not add
   a string registry or broaden unrelated qualified function calls.
4. **RED: evaluated local `String`.** Add the lexical-local form and assert it resolves to the
   same declared operation/row and evaluates to the exact string value.  Retain a non-`String`
   local negative case.
5. **GREEN: preserve typed local value.** Generalize the bounded TASK-2015 value transport only
   to the `String` value/atom needed by this operation; unsupported expressions or values must
   reject at their existing owner boundary.
6. **RED: metadata, admission, and safe direct dispatch.** Add a controlled provider fixture
   implementing the validated exact `FsProvider` operation metadata and fixed path/content map.
   Prove row-only, absent binding, and metadata/signature mismatch fail before invocation; prove
   the matching explicit binding dispatches exactly once and returns fixed content without reading
   the host filesystem.
7. **GREEN: exact provider mapping.** Add the precise `PosixFs::read` provider-operation metadata
   and bind it through the existing typed registration path.  Keep all production filesystem
   behavior unchanged unless a separately admitted provider is explicitly registered.
8. **RED/GREEN: checked CPS String `Raise`.** Extend the private inspection lowering and atom
   conversion so both literal and checked-local forms produce exact `Raise` plus
   `Atom::String("fixture-path")`; add a negative that unsupported value conversion does not get
   stringified or silently lowered.
9. **Regression and evidence.** Run the focused task test and prior TASK-2010/2011/2012/2015
   tests, relevant typechecker tests, `cargo fmt --check`, affected all-target Clippy with warnings
   denied, docs/traceability validation, and `git diff --check`.  Only after green implementation
   evidence, update this task status, `CHANGELOG.md`, `PLAN-INDEX.md`, and traceability links to
   `TYPE-TARGET-ROW-001`, `LOWER-SURFACE-CORE-001`, `SEM-EFFECT-RAISE-001`, and exercised
   `SEM-EFFECT-LOOKUP-001`.

## Expected Completion Evidence

- An ordinary source fixture passes `Engine::parse` and `Engine::check` for both string-literal
  and checked-local path forms and exposes exactly `PosixFs::read`, not dotted metadata or a raw
  source string, in its requirement row.
- A controlled deterministic provider proves exact binding/admission and one string-argument
  dispatch; it must be impossible for the test to pass by reading an arbitrary local file.
- Negative tests establish reject-before-dispatch for missing/mismatched authority and retain the
  removed `invoke` syntax failure.
- Private inspected CPS terms contain canonical identity, `[String] -> String`, local row, and
  `Atom::String`; no test describes this as production CPS execution or handler realization.
- Changelog, task/index status, and semantic traceability link only the tested narrow behavior;
  docs gates and source verification commands are green.

## Completion Checklist

- [x] Literal and checked-local `String` forms resolve through one local declaration-backed
  `PosixFs::read` identity and signature.
- [x] The canonical non-granting row is exact, deduplicated, and not authority.
- [x] `FsProvider` metadata and explicit binding are exact and validated; missing/mismatched cases
  reject before dispatch.
- [x] A deterministic controlled provider returns fixture content with one exact String dispatch;
  no arbitrary host filesystem read is introduced.
- [x] Private CPS inspection preserves String atom, identity, signature, and row without a
  production-CPS claim.
- [x] `invoke` remains absent/rejected, with no string or row-text fallback.
- [x] Focused regressions, formatting, Clippy, docs/traceability, and diff gates pass.
- [x] Completion documentation and changelog describe only the implemented/tested boundary.

## Completed `PosixFs::read` Slice

Normal `Engine::check` declaration resolution now retains the local nominal `PosixFs` identity
when registering local declarations, so `impl Fs<PosixFs> { read(path) = path }` resolves the
ordinary source forms `PosixFs::read("fixture-path")` and `let path = "fixture-path";
PosixFs::read(path)`. Both forms retain exactly the declared `String -> String`
`DeclaredConcreteOperation` identity and one non-granting `PosixFs::read` row item. Resolution
does not derive identity from a local name, a provider spelling, or a row string; wrong argument
types and unknown operations fail during normal checking.

The controlled fixture provider is selected only through a registered exact binding whose metadata
names the complete declared operation and its required `PosixFs.read` row. The admitted binding
dispatches once with `Value::String("fixture-path")` and returns fixed fixture content without
performing host I/O. Missing bindings, provider-operation mismatch, and required-row mismatch all
fail closed before provider dispatch. Existing interface-qualified dispatch precedence is retained;
this slice does not introduce any fallback that matches an interface, operation tail, row text, or
legacy source `invoke` spelling.

Private checked-CPS inspection of either source form produces `Raise(PosixFs::read,
[Atom::String("fixture-path")], {PosixFs::read})` with the declared `String -> String` signature.
This is an inspection artifact only, not production Core/CPS execution or handler realization.
The completed boundary excludes imports, generics, handlers, production Core/CPS, and actual
`FsProvider` filesystem reads.

Focused evidence is
[`task_2017_posixfs_read_symbolic_operation.rs`](../../../crates/ash-engine/tests/task_2017_posixfs_read_symbolic_operation.rs)
(9/9): it covers literal and lexical-local identity/row preservation, wrong argument and unknown
operation rejection, provider metadata and row mismatch rejection, missing-binding
reject-before-dispatch, one controlled String dispatch without host I/O, and String-atom `Raise`
inspection. Related focused regressions remain green: TASK-2010 (5/5), TASK-2011 (6/6),
TASK-2012 (8/8), TASK-2015 (2/2), TASK-2001 (10/10), and `ash-typeck` (477/477). The affected
engine all-target/all-feature Clippy gate with warnings denied, formatting, and `git diff --check`
are clean.
