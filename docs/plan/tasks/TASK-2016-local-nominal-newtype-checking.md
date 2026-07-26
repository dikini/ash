# TASK-2016: Local Nominal Newtype Checking

**Status:** Complete — normal program checking registers local non-generic newtypes before body
checking, enforces their sole tuple constructors and nominal distinction, and rejects opaque,
recursive, and colliding declarations deterministically. Runtime representation/execution and
cross-module or generic semantics remain deferred.
**Phase:** Implementation follow-up from
[TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md)

## Description

Realize the already accepted local, non-generic `newtype` semantics on the normal
`Engine::parse` / `Engine::check` program path.  A declaration such as
`newtype OrderId = OrderId(Int);` introduces a fresh nominal type and a sole
value-level tuple constructor.  The constructor accepts exactly one value of
its declared representation type and produces `OrderId`; `OrderId` neither
coerces to `Int` nor receives an implicit coercion from it.

This task is a typechecking realization of the target contract, not a newtype
design decision and not a runtime-representation claim.  It must integrate
local declarations into the normal program `TypeEnv`, rather than relying only
on the earlier declaration-registration or module-summary evidence in
TASK-2001.

## Authoritative References

- [SPEC-095b §6.7](../../spec/SPEC-095b-TARGET-GRAMMAR.md): canonical
  `newtype` grammar, explicit constructor, nominal contrast with transparent
  aliases, and inhabited-representation requirement.
- [SPEC-097b §3.9](../../spec/SPEC-097b-TARGET-TYPE-SYSTEM.md): fresh nominal
  identity, explicit wrapper constructor, and no automatic representation
  coercion.
- [TASK-2001](TASK-2001-target-grammar-gap-and-spec-conflict-decision.md):
  parser/AST admission, declaration metadata, and the explicit remaining
  integrated newtype-checking gap.

## Scope

### In scope

- Register every module-local, non-generic `NewtypeDef` on the ordinary
  program-checking `TypeEnv` path before callable bodies are checked.
- Treat a newtype name as a fresh nominal type identity, not a transparent
  alias of its representation.
- Register exactly its declared tuple-style value constructor.  Validate one
  payload against the representation and give a successful constructor
  application the fresh newtype result type.
- Reject both implicit directions of representation/newtype coercion and
  reject interchange between two distinct wrappers with the same
  representation.
- Reject a newtype representation that resolves to a bodyless opaque nominal
  declaration as uninhabited, with a deterministic diagnostic that names the
  representation.
- Add focused ordinary-program tests, semantic traceability only for behavior
  actually implemented, and the required task/plan/changelog evidence.

### Explicit exclusions

- Runtime erasure/layout, interpreter execution, Core representation lowering,
  destructuring/pattern execution, and provider/handler behavior.
- Imports, re-exports, visibility/coherence, cross-module newtype identity,
  generics/phantom parameters, aliases involving newtypes,
  derived impls, unsafe conversions, or any automatic coercion feature.
- Recursive newtypes are not supported: direct and mutual recursive representations reject
  deterministically during normal checking rather than acquiring a partial recursive meaning.
- Parser expansion for bodyless ordinary `type Name;`; this task must preserve
  its current rejection.  The relevant negative representation case is the
  already supported bodyless opaque `builtin type Name;` boundary.

## Requirements

1. Normal program checking registers the local nominal declaration before
   checking functions that name the wrapper in signatures or bodies.
2. `Constructor(payload)` succeeds only when `Constructor` is the declared
   constructor and `payload` has the declared representation type; the result
   is the declared fresh nominal type.
3. The checker preserves nominal distinction in all ordinary call/result
   compatibility checks: `OrderId != Int`, and separately declared wrappers
   remain distinct even when both wrap `Int`.
4. A representation that is a bodyless opaque nominal type is rejected before
   constructor use, with no fabricated inhabitant or transparent fallback.
5. No fallback identifies a constructor by representation name/text, and no
   alias canonicalization or source spelling weakens nominal equality.
6. Direct or mutual recursive representations and local ordinary-type/newtype
   name-or-constructor collisions reject deterministically; primitive and prelude type names
   cannot be shadowed by a newtype.
7. The slice neither claims nor changes runtime erasure, imports, generics,
   handler semantics, or bodyless ordinary-type parsing.

## TDD Steps

1. **Freeze the existing declaration seam.** Locate the parsed `NewtypeDef`,
   TASK-2001 registration metadata, normal `Engine::check` program setup, and
   ordinary constructor/call typechecking routes.  Confirm that `type Name;`
   remains a parser rejection.
2. **RED: nominal constructor success.** Add an ordinary source-program test
   for `newtype OrderId = OrderId(Int);` whose `main` returns `OrderId(7)` as
   `OrderId`.  This must use `Engine::parse` and `Engine::check`, not a direct
   registration query.
3. **RED: payload and nominal-negative cases.** Add tests rejecting a wrong
   constructor payload, newtype-to-representation use, representation-to-
   newtype use, and passing one `Int`-wrapper where another is required.
4. **GREEN: normal TypeEnv registration and constructor checking.** Implement
   only the registration/typechecking changes required by those tests.  Keep
   the constructor's identity declaration-backed and keep ordinary unification
   nominal.
5. **RED/GREEN: inhabitation boundary.** Add a `builtin type Token;` / `newtype
   Wrap = Wrap(Token);` ordinary-checking rejection, then reject that opaque
   representation deterministically.  Retain a control that bodyless ordinary
   `type Token;` is still parse-rejected.
6. **Regression and evidence.** Run the affected parser/typechecker/engine
   tests plus workspace formatting and Clippy as appropriate.  Update
   `CHANGELOG.md`, `PLAN-INDEX.md`, this record, and semantic traceability only
   after tested implementation exists; run the docs/traceability gate and
   `git diff --check`.

## Completion Checklist

- [x] Local non-generic newtypes register on the normal program `TypeEnv` path.
- [x] The sole declared tuple constructor accepts exactly its representation
  payload and returns the fresh nominal type.
- [x] Wrong payloads, both coercion directions, and sibling-wrapper confusion
  fail with deterministic nominal diagnostics.
- [x] A bodyless opaque representation is rejected as uninhabited; bodyless
  ordinary `type` syntax remains rejected.
- [x] Direct/mutual recursive representations and local declaration collisions reject before
  body checking; primitive/prelude names remain unavailable to newtypes.
- [x] No runtime/layout, import, generic, handler, or parser expansion claim is introduced;
  recursive forms are explicitly rejected rather than partially implemented.
- [x] Focused tests and applicable formatting/Clippy/docs/traceability/diff
  checks pass.
- [x] `CHANGELOG.md`, `PLAN-INDEX.md`, task status, and traceability evidence
  reflect only implemented, tested behavior.

## Evidence and Non-claims

The completion evidence must be an ordinary source program that passes through
`Engine::parse` and `Engine::check`, not merely a `TypeEnv` metadata query or a
module-summary assertion.  It must establish constructor payload checking and
both directions of nominal non-coercion.  The uninhabited case must establish
that a bodyless opaque representation cannot manufacture a wrapper value.

Even after completion, this task will not prove representation erasure or
execution, imported/generic/recursive semantics, pattern behavior, or handler
interaction.  Those require their own task records and evidence.

## Completed Local Nominal Newtype Slice

The normal `Engine::parse` / `Engine::check` path now registers every supported local newtype
before callable signatures and bodies are checked, then resolves and records its representation
before constructor checking. `newtype OrderId = OrderId(Int);` therefore gives `OrderId` a fresh
nominal identity and makes only the declared `OrderId` tuple constructor available: it accepts an
`Int` payload and returns `OrderId`. The checker does not unfold that identity, so `OrderId` cannot
be passed where `Int` is required, `Int` cannot be passed where `OrderId` is required, and two
separately declared `Int` wrappers remain distinct.

The bounded declaration boundary fails closed. A bodyless `builtin type Token;` cannot be used as
the representation of `newtype Wrap = Wrap(Token);`; bodyless ordinary `type Token;` remains a
parser rejection. Direct and mutual recursive representation graphs reject before body checking,
as do ordinary-type/newtype name or constructor collisions in either declaration order and
attempts to shadow primitive/prelude names. These rejections prevent partial identity or
constructor overwrites; they do not provide recursive newtype semantics.

Focused ordinary-program evidence is
[`task_2001_local_nominal_newtype_checking.rs`](../../../crates/ash-engine/tests/task_2001_local_nominal_newtype_checking.rs):
it exercises constructor success, payload mismatch, both nominal non-coercion directions,
sibling-wrapper distinction, opaque-representation rejection, parser preservation, recursion,
local collisions, and primitive-name protection through `Engine::parse` and `Engine::check`.
The trace links this completed checking boundary to the target grammar and type rules only; it
does not claim runtime erasure, execution, imports, generics, pattern behavior, handlers, or
cross-module identity.
