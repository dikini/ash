---
id: spec.ash.surface-to-core-lowering
title: Ash Surface-to-Core Lowering
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-07-03
verified_against:
  specs:
    - docs/spec/SPEC-095b-TARGET-GRAMMAR.md
    - docs/spec/SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md
    - docs/spec/SPEC-097b-TARGET-TYPE-SYSTEM.md
    - docs/spec/SPEC-098b-TARGET-IR.md
    - docs/spec/SPEC-100-CORE-TYPE-CHECKING.md
  audits:
    - docs/audit/2026-06-29-target-spec-notes-gap-audit.md
---

# SPEC-098c: Ash Surface-to-Core Lowering

**Status:** Draft — general lowering bridge from expanded surface AST to Core.
**Scope:** This document specifies how expanded target surface syntax lowers into Core AST plus
sidecars. It complements SPEC-098b, which owns Core/CPS IR carriers.
**Depends on:** SPEC-095c, SPEC-097b, SPEC-098b, SPEC-100.

## 1. Purpose

The target specs previously described grammar and Core/CPS carriers but did not define the general
bridge between them. This spec owns that bridge. Its input is the expanded surface AST from
SPEC-095c, not raw parser syntax. Its output is checked-Core-ready AST plus sidecars for contracts,
evidence, traces, diagnostics, and source origins.

## 2. Pipeline and boundaries

```text
expanded surface AST
  -> name and notation resolution complete
  -> surface type elaboration summaries
  -> Core AST + sidecars
  -> SPEC-100 Core type checking
  -> SPEC-098b Core-to-CPS lowering
```

Lowering assumes:

- macros have expanded;
- macro summary resolution, token-tree reparse, binder hygiene validation, and typed macro checking
  have either completed or rejected;
- notation and operator sections have become calls or closures;
- source origins are preserved;
- row items are normalized but may still contain variables/obligations for type checking;
- facts/evidence/contracts have stable source identities.

## 3. Global invariants

1. **No surface sugar in Core.** Core has calls, values, handlers, rows, traps, and sidecars, not
   custom notation, operator sections, or macro invocations.
2. **Origins survive.** Every Core term and sidecar derived from surface syntax records enough origin
   metadata for diagnostics and audit trails.
3. **Rows remain requirements.** Lowering records row requirements; it does not grant authority.
4. **Contracts are sidecar-backed.** Contract predicates lower to `LoweredPredicate` and runtime
   check plans rather than raw source expressions.
5. **Traces are explicit.** Trace contracts lower to trace/monitor sidecars and event-emission
   points, not hidden handler behavior.

## 4. Callable declarations and rows

A callable declaration lowers to a Core callable summary and body term:

```text
fn f(params) -> {ρ} T where row { W } { body }
  => CoreFn { name: f, params, result: T, row: normalize(ρ, W), body: lower(body), origins }
```

Inline row syntax and `where row` syntax are mutually exclusive at the surface grammar layer but
normalize to the same callable row summary. If both are absent, lowering asks surface type inference
for an inferred row; exported/public callables must receive an explicit or summarized public row.

**Current implementation note (Phase 178 / Phase 182 / Phase 183 / Phase 185 / Phase 188 / Phase 189 / Phase 190 / Phase 191 / Phase 192).** The current bridge preserves explicit inline callable
rows and expanded `where row` rows from parser carriers through engine/typecheck-facing callable
summaries into `CoreType::Function { row, .. }` metadata for local and imported/exported callables.
Rowless callables continue to lower with the default empty Core row. Open row tails are preserved as
Core row tails in this metadata bridge. This is still a requirement-recording bridge: it does not
perform row-polymorphic inference, install operation authority providers, admit roles, register
handlers, or wire runtime authority. Phase 182 additionally keeps row-bearing `fn` bodies that use target
ambient `do { ... }` on this same path: the row remains callable metadata and the body lowers through
ordinary direct-style Core sequencing. Phase 183 classifies the corresponding admission-side
discharge families without changing lowering: operation rows require existing operation authority,
while resource, role, policy, evidence, and failure rows remain distinct requirements. Phase 185
accepts `fn main`-only entry sources by synthesizing an internal runtime adapter that calls the
ordinary `main` function, and Phase 186 routes CLI dry-run for ordinary files through that same
file-backed parse/check path; callable metadata and body lowering still flow through the same
function path, so this does not add a second Core semantic path. Phase 186 also aligned runtime
field projection for named constructor payload values with the surface/Core fixture accepted by this
path. Phase 187 adds structural record expressions to the same path: record fields lower as ordinary
field expressions and evaluate to `Value::Record` without using nominal constructor identities.
Phase 188 keeps ADT constructor-expression match scrutinees on the same ordinary expression path:
the constructor lowers as a normal value expression and the `match` lowers to the existing Core
match representation. Phase 189 extends function-body match scrutinees to ordinary call,
field-projection, and binary expressions without changing the Core match shape. Phase 190 lowers
target `do` expression statements as direct-style sequencing that evaluates the expression and
discards the result before continuing. Phase 191 applies the same direct-style sequencing rule to
ordinary block expression statements.
Phase 192 keeps postfix field projection on record and constructor primary expressions on the
existing `FieldAccess` lowering path. Phase 193 keeps tuple-payload ADT constructors on the
existing constructor lowering path by preserving positional payloads as stable `_0`, `_1`, ...
fields before type checking and Core lowering.

`where row` items lower as follows:

| Surface item | Lowering product |
|---|---|
| operation/resource/role/policy/channel/process/fail item | row requirement |
| `fact name: requires/ensures/...` | fact sidecar with stable id |
| `proof` or proof evidence | discharge/evidence sidecar |
| `evidence path` | row evidence requirement |
| direct `requires {p}` sugar | canonical generated fact/evidence/check artifacts |

## 5. `do` sequencing

Target `do { ... }` lowers to direct-style Core sequencing. Each `let` or direct `<-` binding lowers
to an ordinary Core `Let`; the final `return` lowers to the tail expression. The block itself does
not install authority, select a tower runtime, or wrap its result in `Act`, `Proc`, or `Workflow`.
Rows remain attached to the enclosing callable type/summary and are validated as requirements.

Ordinary block expressions lower through the same Core `Let` spine. A block statement `expr;`
lowers to a discard binding before the remaining block tail; it does not introduce a runtime
profile or computation wrapper.

```text
do { x <- m; return k(x) }
  => let x = lower(m) in lower(k(x))
```

Explicit `do:Act`, `do:Proc`, and `do:Workflow` are deprecated compatibility forms. They are not
the target semantic foundation and must not be used for new development. Any remaining
compatibility lowering must target ordinary row-bearing Core computation without introducing
`Act`, `Proc`, or `Workflow` Core terms, IR nodes, public stdlib types, or runtime entry paths.

## 6. Handlers and provider boundaries

`handle expr with h` lowers to ordinary handler application/installation. The handler value must be
handler-marked by type checking; lowering records the handler origin and emits the Core/CPS handler
shape required by SPEC-098b.

```text
handle expr with h
  => lower_handler_apply(h, lower(expr))
```

`on comp { Impl::op(pat, resume) => body; done(x) => done_body }` lowers to handler clauses with
operation identity `Impl::op`, explicit continuation/resume binding, and a done clause. Resume
multiplicity is checked by the Core/CPS continuation-multiplicity specs.

`derive handler` is a surface synthesis step before final Core lowering. Generated handler clauses
carry `Origin::Desugaring` metadata pointing to the derive site.

Provider frames represent runtime authority at explicit runtime boundaries. Lowering records the
provider boundary; operational dispatch is specified in SPEC-099b. Lowering must not synthesize
provider frames merely because a callable row mentions an operation identity.

## 7. Impl operation identity

Operation identity is the impl type plus operation name. Generic code lowers abstract operation
calls as `F::op`; monomorphization or specialization may later rewrite them to concrete identities
such as `PosixFs::read`.

The canonical concrete identity form is `ImplType::op`.

```text
F.read(path)       => OperationIdentity(F::read)
PosixFs.read(path) => OperationIdentity(PosixFs::read)
```

Any type can serve as an impl identity carrier, including bodyless nominal types and data-carrying
impl types. Lowering must not impose representational restrictions not present in the target specs.

## 8. Facts, evidence, and contracts

A fact declaration lowers to a fact sidecar with a stable identity. Evidence and proof declarations
lower to discharge metadata and evidence requirements. Direct contract row sugar lowers to generated
fact/evidence/check artifacts so the canonical row remains algebraic.

Contract predicate lowering follows NOTE-031/NOTE-033 and SPEC-100:

```text
surface predicate
  -> expanded predicate AST
  -> binder/snapshot environment
  -> PredicateSummary
  -> LoweredPredicate + RuntimeCheckPlan
```

`old(path)` lowers to a boundary-local `SnapshotRef`. Snapshot roots must be declared in the
predicate environment. Predicate evaluator failure lowers to `ContractPredicateFault`; false dynamic
predicates lower to `ContractViolation(ContractDiagnostic)`.

## 9. Trace contracts and monitors

A `trace` row item lowers to a `TraceContract` sidecar plus a monitor plan. Lowering also records the
trace-fact alphabet and event-emission points needed by operational semantics.

```text
trace workflow::commitment
  => TraceContract { id, formula, alphabet, monitor_plan, origin }
```

Trace formulas are not handler clauses. Runtime monitor violations and monitor faults are distinct
operational outcomes in SPEC-099b.

## 10. Macro, notation, and operator-section erasure

Macros must not reach Core. Notation uses lower after expansion as ordinary callable calls. Operator
sections lower to callable values or eta-expanded closures before Core.

If the implementation preserves macro invocation syntax as a parsed surface carrier before that
invocation has been expanded, lowering must reject that carrier explicitly. It must not erase the
invocation, guess an expansion, lower it as an ordinary call, or admit it through public export
summaries.

The Phase 172 parser-first expression macro MVP may expand a narrow local subset before this lowering
boundary: a local `MacroDecl` with a parsed expression template may expand an unqualified
parenthesized `name!(ExprList?)` invocation into ordinary expanded surface expressions. The final
Core product still contains no macro declarations and no macro invocations; implementations may keep
source-side `MacroDecl` templates in an expanded-surface wrapper for diagnostics so long as executable
residual macro invocations are rejected before lowering. Unsupported macro declarations,
bracketed/braced invocations, qualified macro-like paths, missing or duplicate macro names, arity
mismatches, recursive/depth-overflowing expansions, imported macro activation, and binder-introducing
templates must reject before Core. Macro expansion metadata is source-side diagnostic metadata only;
it does not create rows, authority, contracts, failures, proof evidence, or runtime constructs.

Phase 173 extends the pre-lowering macro boundary but does not move macros into Core:

- macro summaries are consumed before lowering and are never encoded as callable summaries;
- imported/exported macro activation must fail closed before export acceptance if a macro summary is
  missing, ambiguous, malformed, or conflicts with a callable export;
- delimiter-preserving token-tree output must reparse through one audited surface boundary before
  lowering, and the reparsed surface must pass the same residual-macro and notation checks as source;
- binder-introducing macro output must carry validated hygiene metadata before any generated binder
  can lower to Core;
- typed macro signatures and bounded inference obligations must be discharged before expansion
  output is accepted as expanded surface.

Lowering must reject any residual `MacroDecl` that is required for execution, any residual
`MacroInvocation`, any token-tree carrier not explicitly reparsed, any unvalidated generated binder,
and any unresolved macro type obligation. It must not erase those carriers, convert them to ordinary
calls, or accept them through public export summaries.

```ash
a <+> b   => combine(a, b)
(a <+>)   => fn (b) -> combine(a, b)
(<+> b)   => fn (a) -> combine(a, b)
(<+>)     => combine
```

The lowered call/closure keeps `Origin::NotationExpansion` or `Origin::OperatorSection` metadata.
Rows and authority requirements come from the resolved target callable; notation cannot erase them.
Generated helper binders introduced by notation or operator-section lowering are surface hygiene
metadata only; they do not create authority, row, contract, failure, or proof evidence.

## 11. Type inference interface

Surface type inference supplies lowering with:

- normalized callable row summaries;
- selected operation identities;
- handler marker evidence;
- notation target resolution;
- operator-section callable types;
- predicate admissibility summaries;
- public/exported summaries.

Core type checking remains annotation-led and fail-closed. If surface inference cannot produce one
of these products, lowering must reject the program before Core.

## 12. See also

- [SPEC-095c: Surface AST, Macro Expansion, and Notation](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md)
- [SPEC-097b: Target Type System](SPEC-097b-TARGET-TYPE-SYSTEM.md)
- [SPEC-098b: Target IR](SPEC-098b-TARGET-IR.md)
- [SPEC-099b: Target Operational Semantics](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md)
- [SPEC-100: Core Type Checking](SPEC-100-CORE-TYPE-CHECKING.md)

## 13. Changelog

- 2026-07-03: Reconciled Phase 183 operation authority model: lowering preserves impl/type-qualified operation identities as requirements and leaves operation/resource/role/policy/evidence/failure discharge to admission/type/runtime phases.
- 2026-07-02: Reconciled Phase 178 source-to-Core callable row bridge: explicit inline and expanded
  callable rows now reach engine summaries and Core function row metadata while remaining
  authority-neutral; row-polymorphic inference and runtime authority wiring remain future work.
- 2026-06-30: Added Phase 173 macro lowering boundary rules for macro summaries, token-tree reparse, binder hygiene validation, and typed macro obligations.
- 2026-06-30: Added Phase 172 parser-first expression macro MVP lowering boundary: supported local macros must expand before Core, while unsupported macro constructs and declarations remain rejected before Core/export/typecheck acceptance.
- 2026-06-30: Clarified Phase 171 fail-closed lowering boundary for parsed macro invocation carriers and authority-neutral generated helper binders.
- 2026-06-29: Created to define expanded-surface-AST-to-Core lowering, including handlers, impl operation identity, facts/evidence, contracts, trace contracts, notation, and operator sections.
