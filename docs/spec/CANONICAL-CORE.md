---
id: spec.ash.canonical-core
title: Ash Canonical Core
kind: semantic-rule-set
audience: [human, agent]
authority: canonical
status: active
stability: alpha
owner: language-semantics
last_verified: 2026-07-24
---

# Ash Canonical Core

**Status:** Active canonical core (TASK-1986)

This document is the compact normative entry point for target Ash.  It selects and reconciles the
target-state rules named below; it does not assert that the current Rust implementation realizes
all of them.  Rule identifiers are stable trace identities, not section numbers or implementation
symbols.

## Scope and authority

The core has eight owners: language vocabulary, target grammar, target types and effects,
Core/CPS syntax, surface-to-Core lowering, operational semantics, runtime observables, and
implementation conformance.  The target documents cited in each rule are supporting source
material.  If a cited document and this core disagree about target authority, this core controls
until a replacement canonical rule is promoted with the same manifest and conformance updates.

Historical workflow/tower claims remain useful rationale only.  In particular,
`docs/SHARO_CORE_LANGUAGE.md`, the workflow-first formalization boundary, and the old parser-to-Core
contract do not define current target semantics.  They are retained through the manifest's typed
supersession links rather than copied into the productive reading path.

## Canonical vocabulary

**Rule `VOCAB-TARGET-OVERVIEW-001`.** Ash is a function-first language whose computations carry
requirement rows.  A computation row states requirements; it does not grant authority.  Authority
is supplied or discharged by the appropriate admitted provider, role, policy, resource, contract,
channel, or evidence boundary.  The removed `Act`, `Proc`, and `Workflow` tower is historical and
is not a source form, Core carrier, public runtime entry path, or compatibility alias.

The vocabulary reconciles the target grammar and type/effect sources
([SPEC-095b](SPEC-095b-TARGET-GRAMMAR.md), [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md), and
[SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md)).

## Target grammar

**Rule `GRAM-TARGET-MODULE-001`.** A target executable is an ordinary `fn main` declaration; there
is no target `workflow` declaration.  Function signatures use the target row-bearing function
form, and direct-style sequencing is expressed with `do { ... }` and `return`.  Macros and
user notation preserve source structure through expansion and become ordinary callable forms
before Core lowering.

The complete productions and source-preservation boundary are
[SPEC-095b](SPEC-095b-TARGET-GRAMMAR.md) and
[SPEC-095c](SPEC-095c-SURFACE-AST-MACROS-AND-NOTATION.md).  This rule owns target grammar
selection, not parser recovery or currently accepted compatibility syntax.

## Target types and effects

**Rule `TYPE-TARGET-ROW-001`.** Type checking treats a computation row as a normalized set of
requirements.  A computation is admissible only when each required item is discharged by the
ambient effect environment using that item's kind-specific rule.  Row aliases and groups affect
presentation and diagnostics; neither creates authority.  Surface elaboration, Core checking, and
runtime admission remain distinct relations.

This rule reconciles [SPEC-096b](SPEC-096b-TARGET-EFFECT-SYSTEM.md),
[SPEC-097b](SPEC-097b-TARGET-TYPE-SYSTEM.md), and
[SPEC-100](SPEC-100-CORE-TYPE-CHECKING.md).

## Core and CPS syntax

**Rule `CORE-CPS-SYNTAX-001`.** The formal pivot is a Core/CPS calculus with values, tail terms,
function and continuation types, closed rows, and structured traps.  The kernel tail vocabulary
is `LetVal`, total `LetPrim`, `LetCont`, `LetContCall`, `Jump`, `Call`, `If`, `Match`, `Return`,
and `Trap`.  Continuations have a fixed answer type and affine-use state; labels are control-flow
targets, not data values.  Effect extensions add `Raise`, `Handle`, ordered handler/provider
frames, residual rows, and explicitly tracked continuation multiplicity.

The selected target IR source is [SPEC-098b](SPEC-098b-TARGET-IR.md), constrained by the programme
kernel in [PLAN-202](../plan/PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md#82-kernel-%CE%BBash-cps%E2%82%80).
The exact bounded calculus, staged rule identifiers, theorem statuses, and examples are frozen in
the [λAsh-CPS Calculus](ASH-CPS-CALCULUS.md). That detail document refines this rule; it is not a
second `core-cps.syntax` owner. Older CPS reference pages remain A4 explanations, never alternate
syntax owners.

`Return` has one active reading at this boundary: it is the terminal observation of completed
`λAsh-CPS₀` evaluation, not a direct-style source term, a Core boundary form, or an ordinary CPS
call result. A source-level `return v` lowers through its continuation as `Jump k v`; only the
completed kernel configuration projects to `Return v`. This is the typed reconciliation of the
older SPEC-098b phrase “no direct return”: that phrase continues to govern executable CPS tail
terms, while [λAsh-CPS Calculus](ASH-CPS-CALCULUS.md#mathematical-syntax-and-state) governs the
separate terminal-observation form. The checked CPS projection is prototype evidence only.
TASK-2004 selects a retained-private boundary: current production `Engine` APIs declare the direct
`ash_core::Expr` evaluator rather than Core-to-CPS lowering or checked CPS evaluation. Test-runner
repro metadata that executes this legacy substrate names it explicitly as `ash_core::Expr`; its
compatible `ash_interp_core_expr` substrate string is not a Core Ash or CPS representation claim.
Focused tests execute a literal source result through `Engine::run` and an admitted checked source
body through the same declaration; they are behavioral regression evidence, not independent
non-invocation telemetry or source-to-terminal refinement evidence. They do not promote CPS APIs
or select a parser, Core-lowering, or type/answer implementation. TASK-2005 owns
production-parity evidence; TASK-2006 owns the exported CPS API decision.

## Surface-to-Core handoff

**Rule `LOWER-SURFACE-CORE-001`.** Lowering starts from an expanded, resolved surface AST, not raw
parser text.  It produces checked-Core-ready terms plus origin, contract, evidence, trace, and
diagnostic sidecars.  It removes surface sugar without erasing source origins, normalizes rows
without granting authority, and records unsupported or unchecked boundaries explicitly.

This A2 layer-handoff rule is defined by [SPEC-098c](SPEC-098c-SURFACE-TO-CORE-LOWERING.md).  It
supersedes the workflow-first parser-to-Core narrative for target lowering while preserving that
narrative as historical rationale.

## Operational semantics

**Rule `SEM-TARGET-CORE-CPS-001`.** Target execution relates checked Core/CPS configurations to
values, structured traps, and named external-boundary outcomes.  The kernel relation is
deterministic.  Handler/provider lookup is innermost-first; missing discharge is a structured
terminal outcome rather than ordinary stuckness.  Any remaining provider or helper
nondeterminism is explicit, bounded, and owned by a named external relation.

The target operational source is [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md), read with
the Core/CPS vocabulary above.  Workflow-first semantics are historical evidence only and cannot
re-enter the target default path.

## Runtime observable handoff

**Rule `OBS-TARGET-PROJECTION-001`.** Runtime realization refines the Core/CPS semantics through a
terminal observable projection.  The projection distinguishes normal return, structured trap,
pre-entry failure, and explicitly bounded external outcomes; it does not expose Rust storage,
scheduler, or provider internals as language truth.  Observable result fields and any permitted
nondeterminism must be stable enough for conformance cases to compare.

This A2 boundary draws its target state from `SEM-TARGET-CORE-CPS-001` and the observable portions
of [SPEC-099b](SPEC-099b-TARGET-OPERATIONAL-SEMANTICS.md).  [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md)
and the older runtime handoff are retained as historical migration evidence, not target owners.

## Implementation conformance

**Rule `CONF-IMPLEMENTATION-001`.** An implementation conforms only by mapping each tested
behavior to a stable canonical rule and comparing its permitted observables.  Code and tests are
realization evidence, not normative semantic owners.  A case records its rule identity, input,
expected terminal projection or allowed outcome set, and any explicit gap, assumption, or
exclusion.

The target conformance boundary refines the target operational and observable rules above.  The
older [SPEC-026](SPEC-026-IMPLEMENTATION-CONFORMANCE.md) remains a migration-era conformance
source; its workflow-first authority hierarchy is superseded by this rule.

## Traceability evidence

The [semantic traceability graph](SEMANTIC-TRACEABILITY.json) is the reproducible evidence ledger
for these rules and the `λAsh-CPS` detail rules.  It is not a ninth semantic owner: this core and
its canonical rule anchors remain normative.  The graph records implementation and executed-test
evidence separately from deferred or assumed proof obligations, then generates both specification
and implementation coverage reports under `docs/plan/audits/TASK-1990-semantic-traceability/`.
Its validation command and fail-closed orphan policy are maintained with TASK-1990.

## Default reading paths

The manifest generates the productive paths.  Both audiences begin with this core and exclude A5
plans, audits, research, historical sources, and archive material by default:

1. **Human:** authority and vocabulary; grammar; types/effects; Core/CPS; lowering; operational
   semantics; observable handoff; conformance.
2. **Agent:** authority and vocabulary; the relevant canonical rule; any required A2 handoff;
   the linked A3 conformance case; then implementation evidence only as realization evidence.

## Change protocol

Changing a rule above requires the manifest owner, dependent handoff, trace edge, conformance case
or explicit no-case rationale, generated paths, and changelog entry to change together.  A
historical document may explain the decision, but cannot become a default source through that
link.
