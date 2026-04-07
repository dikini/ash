# SPEC-025: Small-Step Operational Semantics

## Status: Draft

## 1. Overview

This specification defines the canonical small-step operational semantics for Ash workflows.

It is the stepwise companion to [SPEC-004: Operational Semantics](SPEC-004-SEMANTICS.md). SPEC-004
remains the normative big-step statement of whole-workflow meaning; this document refines that same
meaning into explicit configuration transitions over the canonical workflow IR from
[SPEC-001](SPEC-001-IR.md).

This specification is derived from the accepted small-step design corpus:

- [MCE-005: Small-Step Semantics](../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](../plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](../plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

This document is workflow-first, not expression-first. Pure expressions and pure pattern matching
remain atomic in v1 and are reused from the existing helper/subjudgment structure already accepted
by SPEC-004.

## 1.1 Scope

This document defines:

1. the canonical small-step judgment for workflow execution,
2. the configuration vocabulary carried between steps,
3. the observability split between configuration state and step labels,
4. the blocked/suspended vs stuck distinction,
5. the canonical workflow-form rule inventory for v1 small-step semantics, and
6. the correspondence contract back to SPEC-004 terminal outcomes.

This document does not define:

- a concrete abstract machine,
- a scheduler or fairness theorem,
- queue layout or mailbox storage representation,
- expression-level micro-stepping,
- user-visible `await` syntax, or
- runtime-only supervision APIs as workflow reduction rules.

## 1.2 Relationship to Other Specifications

- [SPEC-001](SPEC-001-IR.md) defines the canonical workflow forms reduced here.
- [SPEC-004](SPEC-004-SEMANTICS.md) defines the big-step terminal meaning reconstructed by this
  stepwise model.
- [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) owns user/tool-visible runtime observations;
  this document defines semantic carriers, not the full external reporting surface.
- [SPEC-022](SPEC-022-WORKFLOW-TYPING.md) remains the owner of workflow typing and obligation typing
  constraints; this document only describes dynamic obligation state transitions.

## 2. Semantic Backbone

### 2.1 Ambient Context

The ambient context is:

```text
A ::= (C, P)
```

where:

- `C` is the capability context inherited from SPEC-004,
- `P` is the policy environment inherited from SPEC-004.

This context is ambient/static for the purpose of the reduction relation. It is not reified as a
mutable store in the semantic backbone.

### 2.2 Metavariables and Semantic Carriers

The following metavariables are used throughout this document:

- `A` for the ambient semantic context `(C, P)`,
- `κ`, `κ'` for workflow configurations,
- `μ` for a single-step label,
- `Γ` for the runtime environment,
- `Ω` for obligation state,
- `π` for provenance state,
- `T` for cumulative trace state,
- `ε̂` for cumulative effect-summary state,
- `w` for a residual canonical workflow,
- `v` for a returned value,
- `err` for a rejection owned by the SPEC-004 runtime failure boundary.

These carriers are semantic, not machine-prescriptive. A concrete interpreter may realize them through
frames, heaps, queues, tombstones, handles, or other runtime structures so long as it preserves the
same semantic contract.

### 2.3 Configurations

The canonical v1 workflow configuration domain is:

```text
κ ::= Running(Γ, Ω, π, T, ε̂, w)
    | Returned(v, Ω, π, T, ε̂)
    | Rejected(err, Ω, π, T, ε̂)
```

where:

- `Γ` is the runtime environment,
- `Ω` is the current obligation state,
- `π` is the current provenance state,
- `T` is the cumulative trace prefix,
- `ε̂` is the cumulative effect-summary carrier,
- `w` is the residual canonical workflow from SPEC-001,
- `v` is a returned value,
- `err` is a runtime rejection owned by the SPEC-004 failure boundaries.

This vocabulary deliberately reuses the semantic carriers already fixed by SPEC-004 rather than
introducing a generic mutable store `S`.

### 2.4 Step Labels

The transition judgment is labeled:

```text
A ⊢ κ —μ→ κ'
```

with:

```text
μ ::= silent
    | emit(ΔT, δε)
```

where:

- `ΔT` is the trace fragment emitted by the step,
- `δε` is the effect-layer contribution emitted by the step.

Only local step deltas belong in labels. Authoritative cumulative state remains in `κ`.

### 2.5 Observability Split

This specification adopts the MCE-005 split between cumulative semantic state and local observables.

Cumulative state lives in configurations:

- obligations in `Ω`,
- provenance in `π`,
- cumulative trace in `T`,
- cumulative effect summary in `ε̂`,
- terminal success/rejection in `Returned(...)` / `Rejected(...)`.

Local per-step deltas live in labels:

- emitted trace fragment `ΔT`,
- emitted effect contribution `δε`.

This split is normative. Labels are not a second cumulative carrier.

## 3. Judgment Family

### 3.1 Workflow Small-Step Judgment

```text
A ⊢ κ —μ→ κ'
```

This is the primary small-step reduction relation.

### 3.2 Atomic Expression Judgment

Pure expressions remain atomic in v1 and reuse the pure judgment family from SPEC-004:

```text
Γ ⊢e expr ⇓ v
```

No expression-level micro-step relation is introduced by this document.

### 3.3 Atomic Pattern Judgment

Pattern matching remains atomic in v1 and reuses the pattern judgment family from SPEC-004:

```text
Γ ⊢p pat ⇐ v ⇓ ΔΓ
```

No pattern-level micro-step relation is introduced by this document.

### 3.4 Helper Relations

Runtime-owned or algebraic helpers remain helper relations rather than inlined machine steps. In
particular, v1 preserves helper-owned boundaries for:

- `select_receive_outcome(...)`-style receive-arm selection,
- `combine_parallel_outcomes(...)`-style parallel aggregation,
- guard evaluation,
- policy lookup/application,
- obligation/provenance helper operations,
- spawned-child completion/control observation.

These names are schematic. The semantic requirement is the ownership boundary, not a specific Rust
function name or exact implementation API.

## 4. Terminal Projection and Big-Step Correspondence

The terminal projection back to SPEC-004 is direct.

- `Returned(v, Ω', π', T, ε̂')` reconstructs `Return(v, eff', T, Ω', π')`
- `Rejected(err, Ω', π', T, ε̂')` reconstructs `Reject(err, eff', T, Ω', π')`

where `eff'` is the terminal projection of `ε̂'`.

The correspondence table is:

| SPEC-004 terminal dimension | Small-step carrier |
|---|---|
| returned value / rejection error | terminal configuration form |
| obligation state | `Ω` |
| provenance | `π` |
| trace | `T` plus per-step `ΔT` labels |
| effect result / effect summary | `ε̂` plus per-step `δε` labels |

Repeated small-step transitions must reconstruct the same terminal semantic meaning already owned by
SPEC-004.

## 5. Progress Classification

### 5.1 Terminal

A configuration is terminal iff it is either `Returned(...)` or `Rejected(...)`.

### 5.2 Blocked or Suspended

A configuration is blocked/suspended when it is well-formed and semantically owned by an external
or helper condition, but no progress step is currently available.

Canonical v1 examples:

- blocking `Receive` when no arm is currently selectable,
- helper-owned waiting on mailbox or external input,
- runtime-owned child-completion/control observation boundaries,
- explicit paused/suspended runtime states that remain live rather than terminal.

Blocked/suspended is not an error.

### 5.3 Stuck

A configuration is stuck when:

1. it is not terminal,
2. it is not classified as blocked/suspended, and
3. no reduction rule applies.

Stuckness is not a normal user-visible state. In a correct corpus, dynamic failure should instead be
owned by existing rejection categories such as:

- `PatternBindFailure`,
- `PatternMatchFailure(v)`,
- `RuntimeFailure(reason)`,
- other runtime-owned rejection boundaries already defined by SPEC-004.

The intended trichotomy is therefore:

- terminal,
- progress,
- blocked/suspended,

but not ordinary observable stuckness.

## 6. v1 Atomic Boundaries

The following remain atomic in v1:

1. pure expression evaluation,
2. pure pattern matching and binding,
3. guard evaluation,
4. receive-arm selection,
5. parallel terminal aggregation,
6. obligation/provenance helper operations,
7. spawned-child completion sealing and runtime-owned control observation.

This keeps the semantics workflow-first and avoids overcommitting to a machine design.

## 7. Canonical Rule Inventory

The rule inventory is defined over the canonical workflow forms of SPEC-001.

### 7.1 Terminal and Structural Rules

```text
DONE-TERM | RET-EVAL | RET-RETURN | SEQ-STEP | SEQ-ADVANCE | SEQ-REJECT
```

Intent:

- `Done` reaches a terminal no-op boundary,
- `Ret` atomically evaluates its expression and enters `Returned(...)`,
- `Seq` steps the left workflow until it either returns normally and advances to the right workflow,
  or rejects and propagates rejection.

### 7.2 Binding and Branching Rules

```text
LET-EVAL | LET-BIND | LET-REJECT | IF-COND | IF-TRUE | IF-FALSE |
FOREACH-INIT | FOREACH-STEP | FOREACH-DONE
```

Intent:

- `Let` atomically evaluates the bound expression, then applies canonical pattern binding via
  `Γ ⊕ ΔΓ`,
- binding failure is mapped through the existing runtime failure ownership contract,
- `If` atomically evaluates the condition and chooses the continuation branch,
- `ForEach` evaluates the collection atomically, then iterates one element at a time while
  preserving canonical workflow sequencing.

### 7.3 Capability, Policy, and Obligation Rules

```text
OBSERVE-STEP | ORIENT-STEP | PROPOSE-STEP | DECIDE-STEP | CHECK-STEP |
ACT-STEP | OBLIG-ENTER | OBLIG-EXIT | WITH-ENTER | WITH-EXIT
```

Intent:

- `Observe` performs capability lookup/observation under the existing helper boundary, binds its
  result pattern, and continues,
- `Orient` atomically evaluates its expression and continues,
- `Propose` performs proposal formation under the existing helper contract and continues,
- `Decide` atomically evaluates its decision expression, applies the named policy, then continues or
  rejects,
- `Check` discharges/checks the obligation and continues or rejects,
- `Act` executes under guard/helper-owned action contracts as already defined by SPEC-004,
- `Oblig` and `With` preserve scoped obligation/capability transitions without inventing new runtime
  structure.

### 7.4 Modal and Fallback Rules

```text
MAYBE-PRIMARY | MAYBE-FALLBACK | MUST-STEP | MUST-REJECT
```

Intent:

- `Maybe` steps the primary branch first, then switches to fallback only under the rejection class
  permitted by the canonical contract,
- `Must` preserves the strengthened mandatory-success behavior already owned by SPEC-004.

### 7.5 Receive and Concurrency Rules

```text
RECEIVE-SELECTED | RECEIVE-FALLBACK | RECEIVE-FALLTHROUGH | RECEIVE-BLOCKED |
PAR-STEP | PAR-BRANCH-TERM | PAR-AGGREGATE | PAR-REJECT
```

Intent:

- selected receive arms continue with their bodies,
- wildcard/fallback receive behavior remains explicit,
- blocking receive with no currently selectable arm is blocked/suspended rather than stuck,
- `Par` progresses by branch-local interleaving,
- terminal parallel aggregation remains helper-backed rather than encoded as fake left-to-right
  sequencing,
- concurrent rejection semantics preserve helper-owned behavior rather than imposing sequential
  short-circuiting.

## 8. Concurrency and Determinism Boundary

This specification fixes the semantic stance without choosing a runtime scheduler.

1. `Par` is modeled as interleaving of branch-local progress.
2. Terminal parallel aggregation remains helper-backed.
3. The semantics do not commit to a concrete scheduler, fairness property, queue layout, or machine
   representation.
4. Any residual branch-choice latitude remains helper-owned or scheduler-owned rather than being
   erased by presentation order.

The determinism boundary therefore matches the big-step corpus:

- deterministic where current semantics are deterministic,
- explicitly helper/runtime-owned where the current corpus already leaves latitude.

## 9. Runtime Alignment Notes

This section is informative but grounded in current interpreter/runtime artifacts.

### 9.1 Receive Realization Evidence

The current interpreter’s receive execution path already aligns with the v1 small-step stance:

- non-blocking `Receive` falls through observably when no arm matches,
- timeout or wildcard receive continues through the wildcard arm,
- blocking receive waits for message arrival rather than introducing semantic stuckness.

This is consistent with the canonical `RECEIVE-FALLTHROUGH` and `RECEIVE-BLOCKED` inventory.

### 9.2 Coarse Runtime Outcome State Evidence

The current runtime-side classification surface distinguishes:

- `TerminalSuccess`,
- `Active`,
- `BlockedOrSuspended`,
- `InvalidOrTerminated`,
- `ExecutionFailure`.

That coarse surface is not itself the small-step semantics, but it supports the blocked/suspended vs
terminal distinction required by this specification.

### 9.3 Control and Retained Completion Evidence

Current runtime control-link machinery retains terminal completion observations through
`RetainedCompletionRecord` and preserves coarse outcome state plus conservative slices of:

- terminal result,
- effect summary,
- obligation summary,
- provenance summary.

This is evidence for the correspondence/handoff boundary, not a replacement for the semantic carriers
used in this specification.

### 9.4 Current Parallel Realization Boundary

The current interpreter realizes `Par` by concurrent branch execution followed by aggregate result
collection. This is implementation evidence that terminal aggregation remains a distinct boundary,
which is consistent with the helper-backed `PAR-AGGREGATE` stance defined here.

This specification does not require that exact implementation strategy; it requires only the same
semantic boundary and terminal reconstruction contract.

## 10. Explicit Non-Goals

The following are intentionally not canonical small-step workflow rule families in v1:

- user-visible `await`,
- expression-level micro-stepping,
- surface-syntax concurrency forms not present in SPEC-001,
- runtime-only supervision APIs as reduction rules,
- machine-specific scheduler or queue semantics.

## 11. Conformance Requirements

An implementation conforms to this specification iff:

1. it reduces canonical SPEC-001 workflows by a workflow-first step relation equivalent to the
   judgment backbone here,
2. repeated small-step execution reconstructs the same terminal outcomes specified by SPEC-004,
3. blocked receives and equivalent helper-owned waiting states are classified as blocked/suspended
   rather than ordinary stuckness,
4. pure expressions and pure pattern matching remain semantically atomic in v1,
5. `Par` preserves branch-local interleaving with a distinct terminal aggregation boundary, and
6. user-visible observable behavior remains governed by SPEC-021 rather than by ad hoc runtime
   exposure choices.

## 12. Future Work Boundary

Deferred to later work:

- exact fully formal per-rule inference schemata beyond the rule inventory here,
- concrete runtime machine mapping and queue/tombstone representation,
- proof-oriented small-step / big-step correspondence packaging,
- full interpreter/runtime alignment closeout across all five layers.

These are owned by the downstream MCE-006 / MCE-007 alignment work rather than by this document.
