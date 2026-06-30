---
id: spec.ash.surface-to-core-lowering
title: Ash Surface-to-Core Lowering
kind: spec
audience: [human, agent]
authority: design
status: draft
stability: alpha
owner: language
last_verified: 2026-06-29
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

`where row` items lower as follows:

| Surface item | Lowering product |
|---|---|
| operation/resource/role/policy/channel/process/fail item | row requirement |
| `fact name: requires/ensures/...` | fact sidecar with stable id |
| `proof` or proof evidence | discharge/evidence sidecar |
| `evidence path` | row evidence requirement |
| direct `requires {p}` sugar | canonical generated fact/evidence/check artifacts |

## 5. `do` sequencing

A `do` block lowers to Core sequencing. Each binding contributes its local row; rows compose by row
union while contract summaries compose through predicate-transformer obligations, as specified in
SPEC-097b.

```text
do { x <- m; return k(x) }
  => bind lower(m) as x; lower(k(x))
```

Legacy `do:Act`, `do:Proc`, and `do:Workflow` profiles lower as ordinary `do` plus a row-profile
check. The profiles are semantic anchors over one computation model, not separate Core languages.

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
provider boundary; operational dispatch is specified in SPEC-099b.

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

- 2026-06-30: Added Phase 172 parser-first expression macro MVP lowering boundary: supported local macros must expand before Core, while unsupported macro constructs and declarations remain rejected before Core/export/typecheck acceptance.
- 2026-06-30: Clarified Phase 171 fail-closed lowering boundary for parsed macro invocation carriers and authority-neutral generated helper binders.
- 2026-06-29: Created to define expanded-surface-AST-to-Core lowering, including handlers, impl operation identity, facts/evidence, contracts, trace contracts, notation, and operator sections.
