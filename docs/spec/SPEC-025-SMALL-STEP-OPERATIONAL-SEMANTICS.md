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

## 1.3 Faithfulness and Compatibility Contract

This section freezes what counts as a faithful `SPEC-025`.

`SPEC-025` is normative for the workflow-first small-step presentation introduced here, but it is not
an independent semantic authority. A faithful `SPEC-025` must satisfy all of the following:

1. preserve the accepted small-step backbone from
   [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md),
2. remain compatible with the normative big-step and helper-boundary contracts of
   [SPEC-004](SPEC-004-SEMANTICS.md), and
3. keep all implementation-alignment statements conservative and evidence-based relative to
   [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).

This contract narrows what this specification may claim. It does not reopen accepted MCE-005 design
decisions, redesign the runtime, or promote partial implementation evidence into stronger semantic or
conformance claims.

### 1.3.1 Preserved MCE-005 Semantic Decisions

Relative to accepted [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md), a faithful `SPEC-025`
must preserve all of the following semantically, not merely approximately in prose:

1. workflow-first semantic subject: the primary judgment remains `A ⊢ κ —μ→ κ'` over workflows,
   not an expression-first or machine-first semantics;
2. canonical configuration vocabulary: the v1 configuration domain remains
   `Running(Γ, Ω, π, T, ε̂, w)`, `Returned(v, Ω, π, T, ε̂)`, and `Rejected(err, Ω, π, T, ε̂)` with
   ambient context `A = (C, P)`;
3. state/label observability split: authoritative cumulative state remains in configurations,
   while labels carry only local step deltas such as `ΔT` and `δε`;
4. blocked/suspended vs stuck distinction: waiting on helper-owned or external conditions is not
   semantic stuckness;
5. v1 atomic boundaries: pure expressions, pure patterns, guards, receive selection, parallel
   aggregation, obligation/provenance helpers, and spawned-child completion/control observation stay
   atomic or helper-owned in v1 rather than being micro-stepped here;
6. helper-owned concurrency and aggregation boundaries: `Par` remains interleaving-based at the
   semantic level with helper-backed terminal aggregation rather than being rewritten into fake
   left-to-right sequencing or machine-specific scheduler rules.

`SPEC-025` may restate, organize, or clarify these decisions, but it must not weaken, erase, or
silently replace them.

### 1.3.2 Preserved SPEC-004 Compatibility Constraints

Relative to [SPEC-004](SPEC-004-SEMANTICS.md), a faithful `SPEC-025` must preserve the following
compatibility constraints:

1. terminal outcome reconstruction: terminal small-step configurations must reconstruct the same
   `Return(...)` / `Reject(...)` outcomes already owned by SPEC-004, including the terminal role of
   `Ω`, `π`, trace, and effect summary projection;
2. helper-boundary ownership: helper-backed contracts such as
   `select_receive_outcome(...)` and `combine_parallel_outcomes(...)` remain owned helper
   boundaries rather than being flattened into accidental machine internals or presentation-order
   artifacts;
3. receive blocking/fallthrough semantics: receive-arm selection, fallback, non-blocking
   fallthrough, timeout behavior, and blocking waits must remain compatible with the receive helper
   laws and failure ownership already fixed by SPEC-004;
4. `Par` aggregation and determinism boundaries: `Par` must preserve interleaving-compatible branch
   progress together with helper-backed concurrent aggregation, and must not impose sequential
   short-circuiting that contradicts the existing SPEC-004 concurrent combination contract;
5. spawned-child completion/control ownership boundaries: control authority, terminal completion
   sealing, and retained completion observation remain owned by the existing SPEC-004 runtime/
   supervisor contract, not by new surface syntax or new small-step workflow forms introduced here.

Where SPEC-004 is already normative, this document refines presentation only. It does not create a
second incompatible contract.

### 1.3.3 MCE-006 Runtime-Correspondence Honesty Constraints

Implementation correspondence statements in this document must remain conservative relative to the
frozen evidence packet in [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).

Normatively, this section freezes the honesty boundary rather than restating the full evidence packet.
`SPEC-025` may use runtime correspondence evidence to constrain wording, but it must not promote that
informative evidence into stronger normative semantic facts.

In particular, `SPEC-025` must not claim stronger implementation support than the informative runtime
sections and the later compatibility audit can justify for:

1. cumulative semantic carriers such as `π`, `T`, and `ε̂`;
2. blocked/suspended realization and carrier uniformity;
3. retained completion packaging or terminal payload collation;
4. `Par` realization as a fully explicit branch-step machine;
5. any other helper-owned ownership boundary whose current runtime realization is only partial,
   reconstructed, or weak.

Detailed runtime evidence belongs informatively in §9 and, row-by-row, in
[TASK-426: SPEC-025 Big-Step and Runtime Compatibility Audit](../plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md)
rather than in the normative rule/contract sections.

### 1.3.4 Frozen Non-Goals

This specification does not, by this contract:

1. redesign the runtime or choose a concrete abstract machine;
2. introduce new workflow syntax, including user-visible `await`;
3. add expression-level micro-stepping in v1;
4. state or imply a fairness theorem, scheduler guarantee, or queue-layout contract;
5. overclaim current runtime support for `π`, `T`, `ε̂`, or retained completion packaging;
6. reopen accepted MCE-005 or SPEC-004 semantic decisions.

### 1.3.5 Normative vs Informative Placement

Within `SPEC-025`, the following belong normatively in this specification:

- the judgment backbone and configuration vocabulary;
- the observability split between configuration state and labels;
- the blocked/suspended vs stuck classification;
- the v1 atomic-boundary contract;
- the workflow-form rule inventory and helper-boundary ownership stance;
- the terminal projection back to SPEC-004;
- the preserved compatibility constraints and explicit non-goals frozen in this section;
- conformance requirements stated independently of current interpreter implementation details.

The following belong informatively only:

- current interpreter/runtime evidence;
- implementation examples, carrier realizations, or mapping sketches;
- statements about what the runtime presently realizes directly versus only partially;
- any mention of concrete runtime holder types, registries, or packaging artifacts.

Informative material may illustrate or justify the normative contract, but it does not override that
contract and must remain conservative when implementation evidence is partial.

Unless a later section is explicitly labeled informative, it is normative. In particular, the
runtime-alignment material in §9 is informative-only evidence commentary; it does not add new
semantic rules, new conformance obligations, or stronger implementation claims.

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

## 3. Judgment Family and Helper Ownership

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

### 3.4 Helper Relations and Ownership Boundaries

Runtime-owned or algebraic helpers remain helper relations rather than inlined machine steps. This
document uses helper names to mark semantic ownership boundaries already accepted in SPEC-004 and
MCE-005; it does not require a concrete interpreter to expose identically named Rust functions,
traits, or modules.

Normatively, v1 preserves all of the following helper-owned boundaries:

- receive-arm selection and receive wait/fallthrough classification at the
  `select_receive_outcome(...)`-style boundary;
- `Par` terminal combination and concurrent rejection combination at the
  `combine_parallel_outcomes(...)`-style boundary;
- guard evaluation and guard-owned rejection classification at the existing `SPEC-004` boundary;
- policy lookup/application, capability lookup/application, and proposal formation at their existing
  `SPEC-004` workflow/helper boundaries;
- obligation/provenance updates, joins, and scoped transitions at their existing semantic helper
  boundaries;
- spawned-child completion sealing, retained terminal observation, and control-authority lifecycle at
  the existing runtime/supervisor boundary owned by SPEC-004.

These helpers are semantic boundary markers, not extra workflow syntax and not a commitment to
micro-step their internals here. Where a helper can fail, rejection ownership remains with the
owning workflow/helper boundary and must stay within the existing SPEC-004 failure taxonomy rather
than inventing a new small-step error channel.

### 3.5 Rule-Family Presentation Contract

The uppercase names used later in §7 are normative family markers for the accepted workflow-form
inventory, not full formal inference schemata.

Accordingly:

- `SPEC-025` normatively fixes which workflow-form families must be covered,
- it normatively fixes the semantic intent and ownership boundary of each family,
- it does not require this document to spell out complete premise-by-premise inference rules beyond
  the inventory/intent level accepted from MCE-005,
- it does not require a one-to-one correspondence between these family markers and concrete runtime
  implementation entry points.

## 4. Terminal Projection and Big-Step Correspondence

The terminal projection back to SPEC-004 is direct.

- `Returned(v, Ω', π', T, ε̂')` reconstructs `Return(v, eff', T, Ω', π')`
- `Rejected(err, Ω', π', T, ε̂')` reconstructs `Reject(err, eff', T, Ω', π')`

where `eff'` is the terminal projection of `ε̂'`.

This is a semantic reconstruction claim. It does not by itself imply that the current interpreter
already packages those same terminal dimensions as one direct runtime carrier.

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
- semantically live suspension/wait boundaries that are not terminal and not stuck.

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

This keeps the semantics workflow-first and avoids overcommitting to a machine design. In
particular, pure expressions and pure pattern matching remain atomic subjudgments reused from
SPEC-004 rather than new micro-step families introduced by this document.

## 7. Canonical Rule Inventory

The rule inventory is defined over the canonical workflow forms of SPEC-001.

This section is normative as an inventory-and-intent contract. It fixes the accepted family
coverage and semantic grouping for v1 small-step presentation, while intentionally stopping short
of full formal inference schemata.

### 7.1 Terminal and Structural Rules

Family markers:

```text
DONE-TERM | RET-EVAL | RET-RETURN | SEQ-STEP | SEQ-ADVANCE | SEQ-REJECT
```

Intent:

- `Done` reaches a terminal no-op boundary,
- `Ret` atomically evaluates its expression and enters `Returned(...)`,
- `Seq` steps the left workflow until it either returns normally and advances to the right workflow,
  or rejects and propagates rejection.

### 7.2 Binding and Branching Rules

Family markers:

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

Family markers:

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

The helper boundaries named here are ownership boundaries only. They must remain faithful to
SPEC-004's capability/policy/obligation contracts, but `SPEC-025` does not require any particular
API spelling for them.

### 7.4 Modal and Fallback Rules

Family markers:

```text
MAYBE-PRIMARY | MAYBE-FALLBACK | MUST-STEP | MUST-REJECT
```

Intent:

- `Maybe` steps the primary branch first, then switches to fallback only under the rejection class
  permitted by the canonical contract,
- `Must` preserves the strengthened mandatory-success behavior already owned by SPEC-004.

### 7.5 Receive and Concurrency Rules

Family markers:

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

The `Par` family is therefore not a disguised sequential evaluation order. Presentation order in
this section does not collapse the accepted interleaving semantics into a left-to-right machine.

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

## 9. Informative Runtime Alignment Notes

This section is informative but grounded in current interpreter/runtime artifacts.

Per §1.3, nothing in this section upgrades partial runtime evidence into a stronger semantic or
conformance claim. Where current realization is weak or partial, that limitation is stated directly.

For the explicit row-by-row compatibility audit against [SPEC-004](SPEC-004-SEMANTICS.md) and
[MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md), see
[TASK-426](../plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md).

### 9.1 Receive Realization Evidence

Evidence class: direct for the observed non-blocking/fallback behavior; distributed/partial for the blocked-state carrier story.

Current inspected interpreter evidence supports the following receive-path correspondence claims:

- non-blocking `Receive` falls through observably when no arm matches,
- timeout or wildcard receive continues through the wildcard arm,
- blocking receive waits for message arrival rather than introducing semantic stuckness.

This is consistent with the canonical `RECEIVE-FALLTHROUGH` and `RECEIVE-BLOCKED` inventory.

### 9.2 Coarse Runtime Outcome State Evidence

Evidence class: partial/reconstructed.

Current runtime-side evidence supports a coarse correspondence classification among:

- active residual execution,
- blocked/suspended waiting,
- terminal outcome, and
- invalid/runtime-failure boundaries.

That coarse correspondence surface is not itself the small-step semantics, but it is compatible with
the blocked/suspended vs terminal distinction required by this specification. The current runtime does
not expose that distinction through one uniform first-class result carrier, so this correspondence is
reconstructed rather than directly packaged.

### 9.3 Control and Retained Completion Evidence

Evidence class: mixed — direct for control-authority lifecycle, weak/missing for full retained completion packaging.

Current runtime evidence is strongest for control-authority lifecycle and weaker for retained
completion packaging.

- Control-link lifecycle and terminal invalidation are directly evidenced as runtime-owned
  boundaries.
- Retained completion-payload realization is only partial/weak on the inspected main execution path
  summarized by [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).
- Completion-observation waiting as a distinct runtime carrier is weak/missing on the inspected main
  execution path and should not be overclaimed.
- Accordingly, this specification preserves the SPEC-004 completion/control contract normatively,
  but does not claim that the current interpreter already exposes authoritative retained packaging
  for terminal obligations, provenance, trace, and effect summary as one complete runtime carrier.

This is evidence for the correspondence boundary, not a replacement for the semantic carriers used in
this specification.

### 9.4 Current Parallel Realization Boundary

Evidence class: partial/reconstructed.

Current interpreter evidence shows `Par` being realized by concurrent branch execution followed by
aggregate result collection. This is partial implementation evidence that terminal aggregation remains
a distinct boundary, which is consistent with the helper-backed `PAR-AGGREGATE` stance defined here.

This specification does not require that exact implementation strategy. It requires only the same
semantic boundary and terminal reconstruction contract, and it does not treat the current runtime as
already having a fully explicit branch-step interleaving machine.

Current evidence is strongest for successful child-value collation and weaker for helper-backed
aggregation over cumulative carriers such as `Ω`, `π`, `T`, and `ε̂`. Those stronger cumulative-
state aggregation claims remain partial or missing in current runtime evidence.

### 9.5 Cumulative Carrier Realization Boundary

Evidence class: mixed — partial/reconstructed for `Ω`, weak/missing for `π`, `T`, and `ε̂`.

The semantic backbone of this specification continues to use cumulative carriers `Ω`, `π`, `T`, and
`ε̂` because they are part of the preserved semantic contract shared with
[SPEC-004](SPEC-004-SEMANTICS.md).

Current runtime evidence, however, is uneven:

- `Ω` has a real but distributed runtime story, so correspondence is compatible but reconstructed;
- `π` is not currently threaded as one authoritative mutable execution carrier on the main path;
- `T` is not currently threaded as one authoritative cumulative trace carrier on the main path;
- `ε̂` is not currently threaded as one authoritative cumulative effect-summary carrier on the main
  path.

Accordingly, this specification may normatively preserve those carriers as semantic dimensions while
still stating informatively that present runtime support is weak or missing for several of them.

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
