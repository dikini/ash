# SPEC-025: Small-Step Operational Semantics

## Status: Draft

## 1. Overview

This specification defines the canonical small-step operational semantics for Ash workflows.

It is the docs/spec home for the accepted workflow-first small-step contract. The accepted design
backbone remains [MCE-005](../ideas/minimal-core/MCE-005-SMALL-STEP.md), while current runtime-
correspondence evidence remains in [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md).
This document packages that accepted contract as the stable specification surface without
superseding either of those upstream design/evidence backplanes.

It is the stepwise companion to [SPEC-004: Operational Semantics](SPEC-004-SEMANTICS.md). SPEC-004
remains the normative big-step statement of whole-workflow meaning; this document refines that same
meaning into explicit configuration transitions over the canonical workflow IR from
[SPEC-001](SPEC-001-IR.md).

**Sequential workflow contract**: A single workflow in Ash is sequential. The small-step semantics defined here apply to sequential workflow execution. Concurrency and parallelism are modeled at the system level through multiple communicating workflows. Historical sections and rules referencing `Par` document prior design stages and are not part of the current active language contract.

This specification faithfully packages the accepted small-step corpus:

- [MCE-005: Small-Step Semantics](../ideas/minimal-core/MCE-005-SMALL-STEP.md)
- [TASK-395: Canonical Workflow Small-Step Rule Set and Concurrency Semantics](../plan/tasks/TASK-395-canonical-workflow-small-step-rule-set-and-concurrency-semantics.md)
- [TASK-396: Small-Step / Big-Step Correspondence and MCE-006 Handoff](../plan/tasks/TASK-396-small-step-big-step-correspondence-and-mce-006-handoff.md)

This document is workflow-first, not expression-first. Pure expressions and pure pattern matching
remain atomic in v1 and are reused from the existing helper/subjudgment structure already accepted
by SPEC-004. Current implementation support remains partial for several runtime-correspondence rows,
especially cumulative carriers, retained completion packaging, and fully explicit helper-backed
parallel aggregation; those limitations are preserved honestly in §9 rather than erased by spec
wording.

## 1.1 Scope

This document defines:

1. the canonical small-step judgment for workflow execution,
2. the configuration vocabulary carried between steps,
3. the observability split between configuration state and step labels,
4. the blocked/suspended vs stuck distinction,
5. the canonical workflow-form rule definitions for v1 small-step semantics, and
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
5. v1 atomic boundaries: pure expressions, pure patterns, guards, receive selection, and
   obligation/provenance helpers stay atomic or helper-owned in v1 rather than being micro-stepped here;
6. helper-owned concurrency boundaries (Historical): prior `Par` semantics were interleaving-based at
   the semantic level with helper-backed terminal aggregation; this section documents historical
   constraints for that feature, which is no longer part of the active language contract.

`SPEC-025` may restate, organize, or clarify these decisions, but it must not weaken, erase, or
silently replace them.

### 1.3.2 Preserved SPEC-004 Compatibility Constraints

Relative to [SPEC-004](SPEC-004-SEMANTICS.md), a faithful `SPEC-025` must preserve the following
compatibility constraints:

1. terminal outcome reconstruction: terminal small-step configurations must reconstruct the same
   `Return(...)` / `Reject(...)` outcomes already owned by SPEC-004, including the terminal role of
   `Ω`, `π`, trace, and effect summary projection;
2. helper-boundary ownership: helper-backed contracts such as
   `select_receive_outcome(...)` remain owned helper boundaries rather than being flattened into
   accidental machine internals or presentation-order artifacts;
3. receive blocking/fallthrough semantics: receive-arm selection, fallback, non-blocking
   fallthrough, timeout behavior, and blocking waits must remain compatible with the receive helper
   laws and failure ownership already fixed by SPEC-004;
4. spawned-child completion/control ownership boundaries: control authority, terminal completion
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
4. `Par` realization as a fully explicit branch-step machine (Historical);
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
- the workflow-form rule definitions and helper-boundary ownership stance;
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
- `w` is the residual workflow term being reduced: ordinarily a canonical workflow form from
  SPEC-001, and in §7 also possibly one of the specification-only residual forms introduced there to
  make propagation and terminal structure explicit,
- `v` is a returned value,
- `err` is a runtime rejection owned by the SPEC-004 failure boundaries.

This vocabulary deliberately reuses the semantic carriers already fixed by SPEC-004 rather than
introducing a generic mutable store `S`. The specification-only residual forms named in §7 are
presentation devices for rule definition; they do not enlarge the surfaced canonical IR.

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

No expression-level micro-step relation is introduced by this document. This includes the new
canonical `Expr::Let` form added by the SPEC-001/SPEC-004 expression-let amendment: pure
scope extension evaluates atomically (evaluate `expr`, match `pattern`, extend `Γ`, evaluate
`body`), in contrast to `Workflow::Let` which micro-steps through `LET-EVAL` and `LET-BIND`.

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
  `combine_parallel_outcomes(...)`-style boundary (Historical: prior design stage, not part of active v1 contract);
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

### 3.5 Rule-Definition Presentation Contract

The uppercase names used later in §7 are the canonical rule names for this specification's workflow-
first small-step presentation.

Normatively:

- each named rule in §7 is part of the canonical small-step contract;
- each rule definition fixes its subject configuration form, result configuration form, and any
  helper-owned side conditions needed for that step family;
- helper premises name semantic ownership boundaries only and do not require a one-to-one concrete
  runtime API;
- omitted machine detail is omitted intentionally rather than left undefined: if a step depends on an
  atomic expression, atomic pattern, or named helper boundary, that dependency remains atomic in v1.

### 3.6 Rule Notation and Meta-Conventions

Unless a rule states otherwise, premises are read under one ambient context `A = (C, P)`.

The following meta-conventions are used in §7:

1. `Null` denotes the canonical null value already used by the existing corpus.
2. `T ++ ΔT` and `append_effect(ε̂, δε)` denote the configuration-side incorporation of the local
   label deltas carried by `emit(ΔT, δε)`.
3. If a rule emits `silent`, the cumulative carriers named in its conclusion are preserved exactly as
   written in the resulting configuration.
4. A rule with premise `A ⊢ Running(...) —μ→ κ1` is a propagation rule over one workflow subterm; it
   does not authorize expression-level micro-stepping.
5. Where a helper may produce multiple admitted outcomes, that bounded nondeterminism remains owned by
   the helper contract rather than by presentation order in this document.

The following residual forms are specification-only notation used to make rule shape explicit:

- `RetVal(v)` for the post-expression staging state of `Ret`;
- `LetVal(pat, v, w)` for the post-expression staging state of `Let`;
- `IfVal(b, w_then, w_else)` for the post-condition staging state of `If`;
- `ForEachIter(pat, vs, w_body)` for iterator residual structure;
- `ObligBody(...)`, `ObligBodyRet(...)`, `WithBody(...)`, and `WithBodyRet(...)` for scoped-body
  progress and scoped exit staging;
- `MaybeReject(...)` for fallback-pending modal failure classification;
- `ParState(bs)` for branch-local parallel progress state, with branch entries drawn from running or
  terminal configurations.

`w[Γ']` notation in a rule conclusion means “the same residual workflow `w`, now to be evaluated
under environment `Γ'` carried by the resulting configuration.” It is not syntax substitution and not
a new workflow constructor.

The structural laws for these residual forms are part of the rule-definition contract:

- they are introduced only by the rules in §7 that name them;
- they are eliminated only by the matching terminal, propagation, or scope-exit rules in §7;
- they do not introduce new user-visible workflow constructors or reopen the accepted helper-owned
  boundaries.

These are not surfaced workflow syntax and do not change the canonical IR defined by SPEC-001.

## 4. Terminal Projection and Big-Step Correspondence

The terminal projection back to SPEC-004 is direct.

```text
project(Returned(v, Ω', π', T, ε̂')) = Return(v, eff', T, Ω', π')
project(Rejected(err, Ω', π', T, ε̂')) = Reject(err, eff', T, Ω', π')
```

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

Normatively, any complete small-step execution ending in terminal configuration `κt` must satisfy:

```text
A ⊢ κ0 —μ1→ κ1 —μ2→ ... —μn→ κt
κt terminal
────────────────────────────────────────
project(κt) is the authoritative SPEC-004 outcome for κ0
```

## 5. Frozen State Taxonomy

This section freezes the small-step state taxonomy used across the `SPEC-025` / `SPEC-004` /
TASK-405 correspondence story.

The taxonomy is intentionally semantic first. It does not require one concrete runtime carrier or one
Rust enum layout, but it does fix which distinctions later proof work, conformance work, and runtime
alignment work may rely on.

### 5.1 Taxonomy Classes

The canonical v1 taxonomy is:

| Class | Semantic shape | Owner | Notes |
|---|---|---|---|
| progress transition | `A ⊢ κ —μ→ κ'` | workflow rule or helper boundary named by the rule | a derivable step exists now |
| blocked/suspended waiting | `κ = Running(...)` and a blocking classification judgment is derivable | helper-owned or runtime-owned wait boundary | live, non-terminal, and presently non-progressing |
| terminal success | `κ = Returned(v, Ω, π, T, ε̂)` | workflow semantics | successful completion |
| terminal rejection/failure | `κ = Rejected(err, Ω, π, T, ε̂)` | owning workflow/helper boundary under SPEC-004 failure taxonomy | semantic terminal failure/rejection |
| invalid / inadmissible / runtime-failure boundary | meta-level inadmissibility or helper/runtime misuse, not an additional `κ` constructor | lowering/type admissibility plus first owning runtime/helper boundary | must not be mistaken for blocked/suspended waiting |

These classes are exhaustive for proof-facing classification of v1 executions.

### 5.2 Progress and Progress-Capable Running States

`Progress` is a transition fact, not a fourth terminal form:

```text
progress(κ)  iff  ∃ μ, κ'. A ⊢ κ —μ→ κ'
```

Accordingly, a running configuration may be classified in exactly one of the following semantic ways:

1. progress-capable now,
2. blocked/suspended now, or
3. inadmissible as a semantic state.

There is no separate admitted class of ordinary nonterminal stuck states.

### 5.3 Blocked/Suspended Waiting

A configuration is blocked/suspended iff all of the following hold:

1. `κ = Running(Γ, Ω, π, T, ε̂, w)` for some admitted carriers,
2. no transition `A ⊢ κ —μ→ κ'` is currently derivable,
3. the lack of progress is owned by an explicit helper or runtime wait boundary already admitted by
   the semantics, and
4. the state remains semantically live rather than terminal or invalid.

Canonical v1 examples:

- blocking `Receive` when `select_receive_outcome(...) ↝ Blocked`,
- helper-owned mailbox/external wait conditions,
- runtime-owned control/completion observation waits that are semantically live but not themselves new
  workflow forms.

Blocked/suspended waiting is therefore not:

- a rejection,
- a malformed state,
- a silent terminal state, or
- evidence that the semantics are stuck.

### 5.4 Terminal Success and Terminal Rejection/Failure

Terminality is configuration-shaped and final:

```text
terminal(κ) iff κ = Returned(...) or κ = Rejected(...)
```

The success/failure split is preserved explicitly:

- `Returned(...)` is terminal success,
- `Rejected(err, ...)` is terminal rejection/failure,
- the rejection class `err` remains owned by the same `SPEC-004` failure taxonomy already frozen for
  policy, obligation, guard, pattern, terminal-control, and runtime failures.

This preserves the distinction required by `SPEC-004`: terminal semantic failure is still terminal,
but it is not the same thing as blocked/suspended waiting or meta-level inadmissibility.

### 5.5 Invalid, Inadmissible, and Runtime-Failure Boundaries

`SPEC-025` deliberately separates three notions that were previously easy to blur together:

1. inadmissible state: a malformed or out-of-contract would-be configuration/helper input that is not
   part of the admitted semantic execution space;
2. runtime failure: a rejection class already owned by `SPEC-004`, usually surfaced as
   `Rejected(RuntimeFailure(reason), ...)` once the first owning boundary classifies it;
3. terminally unusable runtime/control state: a runtime-observable condition such as invalid or dead
   control authority, which belongs to runtime correspondence surfaces rather than to a new small-step
   configuration constructor.

Normatively:

- inadmissible states are not blocked/suspended,
- inadmissible states are not ordinary observable progress states,
- if such a condition crosses into semantic evaluation, the first owning boundary must classify it
  under the existing `SPEC-004` rejection taxonomy rather than leaving the semantics stuck.

`Stuck` is therefore a proof-failure word, not an admitted runtime/classification target.

### 5.6 Correspondence to TASK-405 Runtime Classification

TASK-405 introduces the coarse runtime surface `RuntimeOutcomeState` with classes such as `Active`,
`BlockedOrSuspended`, `TerminalSuccess`, `ExecutionFailure`, and `InvalidOrTerminated`.

That runtime surface is compatible with, but coarser than, the semantic taxonomy above:

| Semantic taxonomy here | Compatible TASK-405 runtime surface |
|---|---|
| progress-capable `Running(...)` | `Active` |
| blocked/suspended waiting | `BlockedOrSuspended` |
| `Returned(...)` | `TerminalSuccess` |
| `Rejected(err, ...)` | usually `ExecutionFailure`, subject to runtime-side coarse packaging |
| unusable runtime/control state outside an admitted semantic running/terminal configuration | `InvalidOrTerminated` |

This is a compatibility claim, not a bijection claim. The runtime surface is intentionally coarser and
runtime-facing; the semantic taxonomy remains the normative proof language.

## 6. v1 Helper Contract Package

This section packages the helper-owned boundaries that remain atomic in v1 as explicit contracts.

The helper names below are semantic ownership markers. They do not require one concrete Rust API,
trait set, or machine decomposition.

### 6.1 Pure Expression, Pattern, and Guard Boundaries

These remain atomic in v1.

`Γ ⊢e expr ⇓ v`

- Input domain: admitted pure/canonical expressions and environment `Γ`.
- Output domain: one value `v`.
- Ownership of failure/blocking/terminality: no independent blocking or terminality; dynamic failure is
  owned by the enclosing workflow/helper boundary rather than by a second expression-level rejection
  channel.
- Determinism vs bounded nondeterminism: deterministic for fixed helper results, as already frozen by
  `SPEC-004`.
- Preserved semantic dimensions: preserves `Ω`, `π`, `T`, and `ε̂` directly because it does not own
  their transition.

`Γ ⊢p pat ⇐ v ⇓ ΔΓ`

- Input domain: admitted patterns, matched value `v`, and environment `Γ`.
- Output domain: one fresh binding delta `ΔΓ`.
- Ownership of failure/blocking/terminality: no independent blocking or terminality; bind/match
  failure classification is owned by the first enclosing workflow/helper boundary.
- Determinism vs bounded nondeterminism: deterministic on admissible patterns.
- Preserved semantic dimensions: preserves `Ω`, `π`, `T`, and `ε̂`; changes only the continuation
  environment through right-biased extension `Γ ⊕ ΔΓ`.

`evaluate_guard(...)`-style guard evaluation

- Input domain: the guard-owned action/context inputs already fixed by `SPEC-004`.
- Output domain: success/permit to continue, or a guard-owned failure classification.
- Ownership of failure/blocking/terminality: guard rejection remains owned by the `ACT` boundary;
  guard evaluation does not own waiting or terminal states of its own.
- Determinism vs bounded nondeterminism: deterministic for fixed inputs.
- Preserved semantic dimensions: preserves all cumulative carriers except for any rule-local updates
  explicitly attributed to the owning `ACT` step.

### 6.2 Capability, Observation, Action, and Proposal Boundaries

This family covers capability lookup/application and proposal formation boundaries already frozen by
`SPEC-004`, including `observe_capability(...)`, `eval_args(...)`, `lookup_provider(...)`, and
`form_proposal(...)`-style helpers.

**Split Dispatch Contract:**

The ACT execution boundary implements explicit two-phase dispatch:

1. **Provider Lookup**: `lookup_provider(C, provider_name) ↝ provider`
   - Resolves provider name to implementation via capability context `C`
   - Lookup failure maps to `RuntimeFailure(reason)`

2. **Action Dispatch**: `provider.execute(action_name, values) ↝ v`
   - Resolved provider handles action dispatch locally
   - Provider receives action name and evaluated argument values only

This removes the previous overload where one name was used for both lookup and dispatch.

- Input domain: ambient capability context `C`, policy context `P` where applicable, the current
  environment `Γ`, and any operation-specific value/action/provenance inputs.
- Output domain: either an operation-specific success payload sufficient for the owning workflow rule
  (`ObserveOk(...)`, provider value from `execute(...)`, `ProposalOk(...)`, etc.) or an owning-boundary
  failure classification.
- Ownership of failure/blocking/terminality: lookup/provider/runtime misuse maps at the first owning
  boundary to existing `SPEC-004` runtime failures; these helpers do not independently classify
  blocked/suspended waiting or terminal success.
- Determinism vs bounded nondeterminism: lookup-style parts are deterministic; provider/action
  execution may be runtime-defined and therefore nondeterministic exactly where the underlying
  provider behavior is permitted to vary.
- Preserved semantic dimensions: may update exactly the dimensions admitted by the owning workflow rule
  (`Γ`, `Ω`, `π`, `T`, `ε̂`) but may not invent new semantic dimensions or bypass policy/guard
  ownership boundaries.

### 6.3 Policy Decision and Rejection Ownership

This family covers `policy_decision(...)`, `policy_check(...)`, and the `apply_policy(...)`-style
small-step boundary used by `DECIDE`.

- Input domain: policy environment `P`, named policy/action handle, subject value or action context,
  and the current semantic carriers required by the rule.
- Output domain: either permit/success data for the owning workflow step or denial/runtime-failure
  information to be reified by that step.
- Ownership of failure/blocking/terminality: denial is owned by the policy boundary and reconstructs
  `PolicyViolation(...)`; missing/unusable runtime policy state maps to `RuntimeFailure(reason)` at
  the same owning boundary; policy helpers do not own blocked waiting or terminal success on their
  own.
- Determinism vs bounded nondeterminism: deterministic for fixed policy context and inputs.
- Preserved semantic dimensions: preserves the workflow-first terminal projection contract, may update
  `Ω`, `π`, `T`, and `ε̂` only where the owning step explicitly says so, and does not alter the
  blocked/suspended taxonomy.

### 6.4 Obligation Transition, Discharge, and Scoped Reconciliation Ownership

This family covers `check_obligation(...)`, `discharge(...)`, `enter_obligation_scope(...)`,
`leave_obligation_scope(...)`, and the obligation/provenance reconciliation helpers used by scoped
forms.

- Input domain: current obligation state `Ω`, provenance `π` where scoped contracts require it,
  current environment `Γ`, named obligation/role identifiers, and any inner-scope terminal state being
  reconciled.
- Output domain: satisfaction/transition results such as `Satisfied(...)`, updated `Ω'` / `π'`, or
  scoped-entry/scoped-exit carrier deltas.
- Ownership of failure/blocking/terminality: unmet obligations are classified at the owning `CHECK` or
  scope-exit boundary as `ObligationViolation(...)`; malformed runtime obligation state maps to
  `RuntimeFailure(reason)`; obligation helpers do not own blocked waiting, but they do own whether a
  scoped body may discharge or reconcile obligation state on terminal exit.
- Determinism vs bounded nondeterminism: deterministic for fixed inputs.
- Preserved semantic dimensions: preserves the identity of non-mentioned obligations, preserves the
  same terminal returned value/rejection owned by the surrounding rule, and may change only the
  obligation/provenance dimensions explicitly admitted by the scoped transition contract plus any
  emitted `ΔT` / `δε` required by that transition.

### 6.5 Receive-Arm Selection and Waiting Ownership

This family covers `select_receive_outcome(...)` as the atomic owner of receive-arm selection,
fallback/fallthrough classification, receive-owned rejection, and blocked waiting.

- Input domain: receive mode, optional control selector, lowered arm set, environment `Γ`, and the
  current semantic state needed for receive-side deltas (`Ω`, `π` in `SPEC-025`; source scheduling
  modifier and receive runtime context in `SPEC-004`).
- Output domain: exactly one admitted receive outcome such as `Selected(...)`, `Fallback(...)`,
  `Fallthrough(...)`, `ReceiveReject(...)`, or `Blocked` where the mode permits waiting.
- Ownership of failure/blocking/terminality: this helper owns the distinction between immediate
  fallthrough, fallback, waiting, and receive-owned rejection; it does not itself produce terminal
  success/failure configurations, but its result determines whether the enclosing receive step
  progresses, remains blocked/suspended, or rejects.
- Determinism vs bounded nondeterminism: deterministic once the scheduler/source choice is fixed;
  bounded nondeterminism is admitted only where the source scheduling modifier/runtime selection law
  permits multiple valid eligible sources.
- Preserved semantic dimensions: preserves pattern-before-guard ordering, message-consumption timing,
  fallback/fallthrough laws, and the distinction between blocked waiting and rejection. It may update
  only the deltas explicitly owned by receive selection (`ΔΓ`, `ΔΩ`, `Δπ`, `ΔT`, `δε`).

### 6.6 Parallel Branch Progress and Terminal Aggregation Ownership (Historical)

> **Note**: This section documents the prior `Par` workflow form which is no longer part of the active Ash language contract. The content is preserved for historical reference.

This family covers the already-frozen helper-backed `Par` boundary, especially
`combine_parallel_outcomes(...)` together with the branch-state packaging used by `ParState(bs)`.

- Input domain: the parent running carriers plus the terminal branch-state collection `bs` produced by
  branch-local progress.
- Output domain: exactly one admitted aggregate terminal result, either `ParallelReturn(...)` or
  `ParallelReject(...)`.
- Ownership of failure/blocking/terminality: branch-local progress is owned by the ordinary small-step
  rules for each branch; aggregate terminal classification, concurrent rejection combination, and
  terminal carrier collation are owned by `combine_parallel_outcomes(...)`; blocked waiting is not
  introduced by the aggregation helper itself.
- Determinism vs bounded nondeterminism: branch-choice/interleaving before all-terminal is scheduler-
  owned; once terminal branch outcomes are fixed, aggregation is deterministic except for bounded trace
  interleaving latitude already admitted by `merge_traces(...)`.
- Preserved semantic dimensions: preserves branch-local result values, preserves each branch's internal
  trace order, preserves obligation/provenance aggregation laws, preserves the no-sequential-
  short-circuit `Par` stance, and reconstructs the same terminal success/rejection meaning owned by
  `SPEC-004`.

#### 6.6.1 Frozen Branch-Local Carrier Contract

For one running parent configuration:

```text
Running(Γp, Ωp, πp, Tp, ε̂p, Par { workflows })
```

the specification-only residual:

```text
ParState(bs)
```

packages one branch entry per branch. Each branch entry denotes one branch-local execution instance.

Normatively, each branch-local execution instance carries its own branch-local semantic configuration:

```text
Running(Γi, Ωi, πi, Ti, ε̂i, wi)
Returned(vi, Ωi, πi, Ti, ε̂i)
Rejected(erri, Ωi, πi, Ti, ε̂i)
```

with the following ownership boundary:

- `Γi` is branch-local runtime environment state for branch `i`;
- `Ωi` is branch-local obligation state for branch `i`;
- `πi` is branch-local provenance state for branch `i`;
- `Ti` is branch-local cumulative trace for branch `i`;
- `ε̂i` is branch-local cumulative effect-summary state for branch `i`;
- `vi` or `erri` is the branch terminal payload once branch `i` becomes terminal.

The parent carriers `Γp`, `Ωp`, `πp`, `Tp`, and `ε̂p` are not incrementally mutated by `PAR-STEP`.
Instead, `ParState(bs)` owns the branch-local carrier copies or branch-local realizations until the
aggregate terminal step. This freezes the semantic meaning of branch-locality without prescribing one
runtime storage layout.

The required initialization law is:

1. every branch starts from the same parent semantic seed for obligations/effects and an admitted
   branch-local environment/provenance seed derived from the parent;
2. branch-local provenance seeding is helper-owned and must remain compatible with the `fork(...)`
   lineage law from `SPEC-004`;
3. after initialization, later progress of one branch changes only that branch entry in `bs` until
   aggregate terminal collation occurs.

This is the explicit contract that later runtime work must realize: `Par` does not secretly share one
mutable cumulative `Ω` / `π` / `T` / `ε̂` across live branches, and it does not collapse branch-local
progress into fake left-to-right sequential state threading.

#### 6.6.2 Frozen Aggregation Law for `combine_parallel_outcomes(...)`

`combine_parallel_outcomes(Γp, Ωp, πp, Tp, ε̂p, bs)` is defined only when every branch entry in `bs`
is terminal.

Its contract is to combine the branch-local terminal carriers into one enclosing terminal result
without reopening branch execution order.

Normatively:

1. it consumes the terminal branch-local outcomes, not partially-running branch states;
2. it preserves every branch terminal payload exactly;
3. it combines branch-local `Ωi`, `πi`, `Ti`, and `ε̂i` according to the laws below;
4. it reconstructs one enclosing `Returned(...)` or `Rejected(...)` configuration compatible with
   `SPEC-004` `PAR`.

#### 6.6.3 All-Success Aggregation

If every branch is terminal-success:

```text
bs = [Returned(v1, Ω1, π1, T1, ε̂1), ..., Returned(vn, Ωn, πn, Tn, ε̂n)]
```

then:

```text
combine_parallel_outcomes(Γp, Ωp, πp, Tp, ε̂p, bs)
  ↝ ParallelReturn([v1, ..., vn], Ω', π', T', ε̂')
```

with the following laws:

1. result collation preserves branch identity/order by branch index, yielding `[v1, ..., vn]`; this is
   branch-index collation, not evidence of sequential execution order;
2. `Ω'` is the deterministic helper-owned join of the terminal branch-local obligation states, rooted
   in the same parent-originating execution slice and preserving every branch-visible obligation
   transition already made semantically visible;
3. `π'` is the deterministic helper-owned provenance join rooted at the incoming parent provenance seed
   `πp` and preserving every branch lineage as an ancestor;
4. `T'` is the helper-owned trace merge of the terminal branch-local traces together with any admitted
   parent prefix contribution, preserving the internal order of each `Ti` while permitting only the
   already-admitted cross-branch interleaving latitude;
5. `ε̂'` is the helper-owned cumulative effect-summary combination whose terminal projection matches the
   `SPEC-004` all-success effect join law;
6. no branch's local `Ωi`, `πi`, `Ti`, or `ε̂i` may be discarded and replaced by an unrelated parent or
   sibling carrier.

#### 6.6.4 Rejection and Mixed Terminal Aggregation

If one or more branches are terminal rejections:

```text
bs = [..., Rejected(errj, Ωj, πj, Tj, ε̂j), ...]
```

then:

```text
combine_parallel_outcomes(Γp, Ωp, πp, Tp, ε̂p, bs)
  ↝ ParallelReject(err, Ω', π', T', ε̂')
```

where concurrent rejection ownership remains helper-backed.

Normatively:

1. `Par` does not use left-to-right first-failure short-circuit semantics;
2. all branch outcomes that actually reached terminality before aggregation remain semantically relevant
   inputs to the helper, including successful siblings and multiple rejecting siblings;
3. the chosen enclosing `err` is the helper-owned concurrent rejection result admitted by the
   `SPEC-004` `combine_parallel_outcomes(...)` boundary, not a theorem about source-order priority;
4. `Ω'`, `π'`, `T'`, and `ε̂'` are aggregated from the terminal branch-local carriers actually present in
   `bs`, including mixed success/rejection outcomes, according to the same preservation laws as the
   all-success case except that the enclosing terminal class is rejection;
5. no implementation may claim conformance by discarding concurrent terminal sibling carriers merely
   because one branch happened to be observed first.

#### 6.6.5 Blocked, Suspended, and Nonterminal Branches

`combine_parallel_outcomes(...)` has no domain on mixed terminal/nonterminal branch collections.

If some branch entries are still running or blocked/suspended, then the enclosing `ParState(bs)` is
still nonterminal.

Normatively:

1. a blocked/suspended branch remains a live branch-local execution instance carrying its current
   branch-local `Ωi`, `πi`, `Ti`, and `ε̂i`;
2. a blocked/suspended branch is not reclassified as a rejection merely to force aggregate completion;
3. a terminal sibling does not by itself authorize `PAR-AGGREGATE`; the all-terminal precondition is
   required;
4. branch-local waiting preserves already accumulated carrier state exactly as in the general blocked
   taxonomy for `Running(...)` configurations;
5. mixed active/blocked/terminal branch collections remain represented only inside `ParState(bs)` and
   preserve interleaving-compatible future progress.

#### 6.6.6 Helper-Owned Concurrent Outcomes and Nondeterminism Boundary

The following choices remain helper-owned or scheduler-owned rather than fixed by presentation order in
this specification:

- which progress-capable branch steps next;
- which admitted cross-branch trace interleaving is chosen by trace merge;
- which helper-admitted concurrent rejection summary is chosen when multiple branch failures are
  semantically relevant;
- how an implementation internally realizes branch-local carriers before aggregate collation.

The following are not left open:

- branch-local ownership of `Ω`, `π`, `T`, `ε̂`, and terminal payloads during live `Par` evaluation;
- all-terminal as the only admitted aggregation point;
- preservation of each branch's internal trace order;
- the prohibition on collapsing `Par` into fake sequential state threading;
- compatibility with `SPEC-004` `PAR` outcome reconstruction and with the execution-record contract for
  exact terminal carrier meaning.

#### 6.6.7 Implementation Conformance Rule

Two implementations may differ in exact branch execution order, exact branch-step count, or concrete
runtime packaging and still both conform, iff all of the following hold:

1. each branch execution can be reconstructed as preserving its own branch-local `Ω`, `π`, `T`, `ε̂`,
   and terminal payload through the live `Par` interval;
2. any admitted blocked/suspended branch remains nonterminal rather than being collapsed into rejection,
   invalidity, or hidden sequential waiting;
3. once the branch terminal multiset-with-branch-indices is fixed, the implementation's aggregate
   result is one allowed by the helper-backed contract above;
4. each branch trace's internal order is preserved even if the enclosing aggregate trace differs by an
   allowed interleaving;
5. terminal projection back to `SPEC-004` and any claimed execution-record projection remains exact.

Accordingly, semantic equality for `Par` is read modulo admitted branch interleaving and helper-owned
concurrent aggregation latitude, not modulo arbitrary carrier loss or scheduler-specific theorems.

### 6.7 Spawned-Child Completion Sealing and Observation Ownership

This family covers `spawn_runtime(...)`, `seal_completion(...)`, and `supervisor_observe(...)` as the
frozen runtime/supervisor boundary for spawned-child completion.

- Input domain: spawned workflow plus its initial semantic carriers at spawn time, reusable control
  authority, and the child's terminal outcome once one exists.
- Output domain: one runtime-owned control authority at spawn time, one sealed completion payload per
  child terminal outcome, and one observation result projected from that payload.
- Ownership of failure/blocking/terminality: child terminality remains owned by ordinary workflow
  semantics; sealing/observation own exactly the completion-packaging and retained-observation
  boundary; invalid or unusable control/runtime state remains runtime-owned rather than a new workflow
  constructor; live waiting to observe completion is blocked/suspended, not terminal failure.
- Determinism vs bounded nondeterminism: deterministic modulo fresh instance/control identities; once a
  terminal outcome is sealed, repeated observation is deterministic and stable.
- Preserved semantic dimensions: preserves the child's authoritative terminal `result`, `Ω`, `π`, and
  effect-summary projection inside the sealed completion contract, while preserving the non-goal that
  this does not surface a user-visible `await` form or a full trace-as-value transport.

### 6.8 Provenance, Trace, and Carrier-Only Helper Boundaries

This family covers `fork(...)`, `extend_provenance(...)`, `join_provenance(...)`, `merge_traces(...)`,
and analogous carrier-only helpers.

- Input domain: existing carrier values plus helper-specific action/branch inputs.
- Output domain: updated provenance/trace carriers or helper-local deltas.
- Ownership of failure/blocking/terminality: these helpers do not own blocked waiting or terminality;
  misuse/impossible carrier state maps to `RuntimeFailure(reason)` at the owning boundary.
- Determinism vs bounded nondeterminism: deterministic except for freshness generation and bounded
  cross-branch trace interleaving already admitted by the helper laws.
- Preserved semantic dimensions: preserve lineage ancestry, internal branch trace order, cumulative-
  carrier monotonicity, and the requirement that helper-local carrier work not invent new workflow
  results or new rejection classes.

## 7. Canonical Workflow Rule Definitions

The rules below are defined over the canonical workflow forms of SPEC-001.

This section is normative as the canonical rule-definition surface for the accepted workflow-first
small-step semantics. It makes premises, side conditions, propagation structure, and terminal shape
explicit enough for proof and conformance work to cite directly, while preserving helper-owned and
atomic boundaries exactly where the accepted corpus already freezes them.

### 7.1 Terminal and Structural Rules

Canonical rules in this group:

```text
DONE-TERM | RET-EVAL | RET-RETURN | SEQ-STEP | SEQ-ADVANCE | SEQ-REJECT
```

`Done` is the explicit terminal no-op workflow form in canonical IR. Since terminal configurations
are represented only by `Returned(...)` or `Rejected(...)`, the `Done` boundary is projected into
successful terminal completion with the canonical null value.

```text
(DONE-TERM)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Done) —silent→ Returned(Null, Ω, π, T, ε̂)
```

`Ret` remains expression-atomic in v1. The expression premise is not a reducible workflow step; it
is the reused SPEC-004 expression judgment.

```text
(RET-EVAL)
  Γ ⊢e expr ⇓ v
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Ret { expr }) —silent→
      Running(Γ, Ω, π, T, ε̂, RetVal(v))

(RET-RETURN)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, RetVal(v)) —silent→ Returned(v, Ω, π, T, ε̂)
```

`RetVal(v)` is specification-only staging notation for the rule family above. It marks the unique
post-expression, pre-terminal residual state of `Ret` and does not add surfaced IR syntax or reopen
expression micro-stepping.

`Seq` propagates the left workflow until that left side becomes terminal. There is no step from a
right-hand subworkflow until the left side has completed successfully.

**Precondition:** The small-step operational semantics defined here operate over canonical core workflow
forms. Surface statement lists are normatively lowered to nested `LET ... in cont` and `SEQ` forms
before reaching this layer (see [SPEC-002](../SPEC-002-SURFACE.md) §4.4).

```text
(SEQ-STEP)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w1) —μ→ Running(Γ', Ω', π', T', ε̂', w1')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Seq { first = w1, second = w2 }) —μ→
      Running(Γ', Ω', π', T', ε̂', Seq { first = w1', second = w2 })

(SEQ-ADVANCE)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w1) —μ→ Returned(v, Ω', π', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Seq { first = w1, second = w2 }) —μ→
      Running(Γ, Ω', π', T', ε̂', w2)

(SEQ-REJECT)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w1) —μ→ Rejected(err, Ω', π', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Seq { first = w1, second = w2 }) —μ→
      Rejected(err, Ω', π', T', ε̂')
```

Side conditions for this family:

- `SEQ-STEP` applies only when the left-side step remains non-terminal and returns a new residual
  workflow.
- `SEQ-ADVANCE` is the unique sequencing rule that consumes successful completion of the left side;
  the returned value `v` from the left side is not rebound by `Seq`.
- `SEQ-REJECT` preserves rejection ownership from the left-side step; the sequencing form does not
  invent a new rejection category.

### 7.2 Binding and Branching Rules

Canonical rules in this group:

```text
LET-EVAL | LET-BIND | LET-REJECT | IF-COND | IF-TRUE | IF-FALSE |
FOREACH-INIT | FOREACH-STEP | FOREACH-DONE
```

`Let` keeps both expression evaluation and pattern matching atomic.

```text
(LET-EVAL)
  Γ ⊢e expr ⇓ v
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Let { pattern = pat, expr, continuation = w }) —silent→
      Running(Γ, Ω, π, T, ε̂, LetVal(pat, v, w))

(LET-BIND)
  Γ ⊢p pat ⇐ v ⇓ ΔΓ
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, LetVal(pat, v, w)) —silent→
      Running(Γ ⊕ ΔΓ, Ω, π, T, ε̂, w)

(LET-REJECT)
  bind_failure(pat, v) ↝ err
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, LetVal(pat, v, w)) —silent→
      Rejected(err, Ω, π, T, ε̂)
```

Side conditions:

- `bind_failure(pat, v) ↝ err` is schematic notation for the existing SPEC-004 pattern-failure
  ownership boundary; in admissible v1 cases this yields the established pattern-owned rejection
  category rather than a new small-step-only error class.
- `LetVal(...)` is specification-only staging notation analogous to `RetVal(...)`.

`If` atomically decides its branch by evaluating the condition once.

```text
(IF-COND)
  Γ ⊢e condition ⇓ b
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      If { condition, then_branch = w_then, else_branch = w_else }) —silent→
      Running(Γ, Ω, π, T, ε̂, IfVal(b, w_then, w_else))

(IF-TRUE)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, IfVal(true, w_then, w_else)) —silent→
      Running(Γ, Ω, π, T, ε̂, w_then)

(IF-FALSE)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, IfVal(false, w_then, w_else)) —silent→
      Running(Γ, Ω, π, T, ε̂, w_else)
```

`ForEach` atomically evaluates its collection, then iterates one element at a time through explicit
residual workflow structure rather than expression micro-steps.

```text
(FOREACH-INIT)
  Γ ⊢e collection ⇓ List(v1, ..., vn)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      ForEach { pattern = pat, collection, body = w_body }) —silent→
      Running(Γ, Ω, π, T, ε̂, ForEachIter(pat, [v1, ..., vn], w_body))

(FOREACH-STEP)
  Γ ⊢p pat ⇐ v ⇓ ΔΓ
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, ForEachIter(pat, v :: vs, w_body)) —silent→
      Running(Γ, Ω, π, T, ε̂,
        Seq {
          first = w_body[Γ ⊕ ΔΓ],
          second = ForEachIter(pat, vs, w_body)
        })

(FOREACH-DONE)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, ForEachIter(pat, [], w_body)) —silent→
      Running(Γ, Ω, π, T, ε̂, Done)
```

Side conditions:

- `FOREACH-INIT` is defined only for collection results in the admitted list-shaped domain of the
  canonical workflow form.
- Pattern failure for an element is mapped through the same pattern-owned rejection boundary used by
  `LET-REJECT`.
- `ForEachIter(...)` is specification-only residual notation making the iteration structure explicit;
  it does not add surfaced IR forms.
- `w_body[Γ ⊕ ΔΓ]` in `FOREACH-STEP` denotes the same canonical workflow body evaluated under the
  extended environment produced by atomic pattern binding. It is environment-instantiation notation,
  not a new workflow constructor.

### 7.3 Capability, Policy, and Obligation Rules

Canonical rules in this group:

```text
OBSERVE-STEP | ORIENT-STEP | PROPOSE-STEP | DECIDE-STEP | CHECK-STEP |
ACT-STEP | OBLIG-ENTER | OBLIG-STEP | OBLIG-EXIT |
WITH-ENTER | WITH-STEP | WITH-EXIT
```

These rules preserve the accepted helper-owned boundaries from SPEC-004. The workflow rule fixes the
surrounding small-step shape; the helper premise owns the operation-specific internals.

Phase 122 keeps the historical small-step rules as reference material for
existing workflow forms. SPEC-069 owns alpha execution-artifact compatibility:
OODA-shaped examples are library/template/lint surface and must not become
privileged AMIR or bytecode primitive roots.

```text
(OBSERVE-STEP)
  observe_capability(C, capability, Γ, Ω, π) ↝ ObserveOk(v, ΔΩ, Δπ, ΔT, δε)
  Γ ⊢p pattern ⇐ v ⇓ ΔΓ
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Observe { capability, pattern, continuation = w }) —emit(ΔT, δε)→
      Running(Γ ⊕ ΔΓ, Ω ⊗ ΔΩ, π ⊗ Δπ, T ++ ΔT, append_effect(ε̂, δε), w)

(ORIENT-STEP)
  Γ ⊢e expr ⇓ v
  orient_update(Γ, Ω, π, v) ↝ (Γ', Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Orient { expr, continuation = w }) —emit(ΔT, δε)→
      Running(Γ', Ω', π', T ++ ΔT, append_effect(ε̂, δε), w)

(PROPOSE-STEP)
  form_proposal(C, P, action, Γ, Ω, π) ↝ ProposalOk(Γ', Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Propose { action, continuation = w }) —emit(ΔT, δε)→
      Running(Γ', Ω', π', T ++ ΔT, append_effect(ε̂, δε), w)

(DECIDE-STEP)
  Γ ⊢e expr ⇓ v
  apply_policy(P, policy, v, Γ, Ω, π) ↝ Permit(Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Decide { expr, policy, continuation = w }) —emit(ΔT, δε)→
      Running(Γ, Ω', π', T ++ ΔT, append_effect(ε̂, δε), w)

(CHECK-STEP)
  check_obligation(Ω, obligation, Γ, π) ↝ Satisfied(Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Check { obligation, continuation = w }) —emit(ΔT, δε)→
      Running(Γ, Ω', π', T ++ ΔT, append_effect(ε̂, δε), w)

(ACT-STEP)
  evaluate_guard(C, P, provider_name, action_name, args, guard, Γ, Ω, π) ↝ GuardOk
  eval_args(Γ, args) ↝ values
  lookup_provider(C, provider_name) ↝ provider
  provider.execute(action_name, values) ↝ v
  π' = extend_provenance(π, provider_name, action_name, guard, v)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Act { provider_name, action_name, arguments: args, guard, provenance }) —emit(ΔT, δε)→
      Returned(v, Ω', π', T ++ ΔT, append_effect(ε̂, δε))

(ACT-LOOKUP-FAIL)
  evaluate_guard(C, P, provider_name, action_name, args, guard, Γ, Ω, π) ↝ GuardOk
  eval_args(Γ, args) ↝ values
  lookup_provider(C, provider_name) ↝ error reason
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Act { provider_name, action_name, arguments: args, guard, provenance }) —silent→
      Rejected(RuntimeFailure(reason), Ω, π, T, ε̂)
```

**ACT Dispatch Contract:**

The `ACT-STEP` rule implements an explicit two-phase dispatch:

1. **Provider Lookup**: `lookup_provider(C, provider_name) ↝ provider`
   - The capability context `C` maps provider names to provider implementations
   - Lookup failure is handled by `ACT-LOOKUP-FAIL`

2. **Action Dispatch**: `provider.execute(action_name, values) ↝ v`
   - The resolved provider handles the action dispatch locally
   - The provider receives only the action name and evaluated argument values

This split removes the previous overload where one name was used for both provider lookup
and provider-local action dispatch.

Failure-side conditions for this family:

- `observe_capability(...)`, `form_proposal(...)`, `apply_policy(...)`, `check_obligation(...)`,
  `evaluate_guard(...)`, `lookup_provider(...)`, and provider `execute(...)` may fail only through
  their already-owned SPEC-004 rejection boundaries.
- Policy denial remains owned by `DECIDE-STEP` and reconstructs the existing
  `PolicyViolation(policy, v)` category.
- Obligation failure remains owned by `CHECK-STEP` and reconstructs the existing
  `ObligationViolation(obligation)` category.
- Guard failure remains owned by `ACT-STEP` and reconstructs the existing
  `GuardViolation(provider_name:action_name, guard)` category.
- Provider lookup failure is owned by `ACT-LOOKUP-FAIL` and reconstructs `RuntimeFailure(reason)`.
- None of these helpers authorize expression-level micro-steps or runtime-specific machine detail.

Scoped forms preserve helper-owned entry/exit boundaries explicitly.

```text
(OBLIG-ENTER)
  enter_obligation_scope(Ω, π, role, Γ) ↝ (Ω_in, π_in)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Oblig { role, workflow = w }) —silent→
      Running(Γ, Ω_in, π_in, T, ε̂, ObligBody(role, Ω, π, w))

(OBLIG-STEP)
  A ⊢ Running(Γ, Ω_in, π_in, T, ε̂, w) —μ→ Running(Γ', Ω_in', π_in', T', ε̂', w')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω_in, π_in, T, ε̂, ObligBody(role, Ω_outer, π_outer, w)) —μ→
      Running(Γ', Ω_in', π_in', T', ε̂', ObligBody(role, Ω_outer, π_outer, w'))

(OBLIG-CAPTURE-RETURN)
  A ⊢ Running(Γ, Ω_in, π_in, T, ε̂, w) —μ→ Returned(v, Ω_in', π_in', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω_in, π_in, T, ε̂, ObligBody(role, Ω_outer, π_outer, w)) —μ→
      Running(Γ, Ω_in', π_in', T', ε̂', ObligBodyRet(role, Ω_outer, π_outer, v))

(OBLIG-EXIT)
  leave_obligation_scope(role, Ω_outer, π_outer, Ω_in, π_in) ↝ (Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω_in, π_in, T, ε̂, ObligBodyRet(role, Ω_outer, π_outer, v)) —emit(ΔT, δε)→
      Returned(v, Ω', π', T ++ ΔT, append_effect(ε̂, δε))

(WITH-ENTER)
  enter_capability_scope(C, capability, Γ, Ω, π) ↝ (Γ', Ω', π')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, With { capability, workflow = w }) —silent→
      Running(Γ', Ω', π', T, ε̂, WithBody(capability, Γ, Ω, π, w))

(WITH-STEP)
  A ⊢ Running(Γ_in, Ω_in, π_in, T, ε̂, w) —μ→ Running(Γ_in', Ω_in', π_in', T', ε̂', w')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ_in, Ω_in, π_in, T, ε̂,
      WithBody(capability, Γ_outer, Ω_outer, π_outer, w)) —μ→
      Running(Γ_in', Ω_in', π_in', T', ε̂',
        WithBody(capability, Γ_outer, Ω_outer, π_outer, w'))

(WITH-CAPTURE-RETURN)
  A ⊢ Running(Γ_in, Ω_in, π_in, T, ε̂, w) —μ→ Returned(v, Ω_in', π_in', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ_in, Ω_in, π_in, T, ε̂,
      WithBody(capability, Γ_outer, Ω_outer, π_outer, w)) —μ→
      Running(Γ_in, Ω_in', π_in', T', ε̂',
        WithBodyRet(capability, Γ_outer, Ω_outer, π_outer, v))

(WITH-EXIT)
  leave_capability_scope(capability, Γ_outer, Ω_outer, π_outer, Γ_in, Ω_in, π_in)
    ↝ (Γ', Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ_in, Ω_in, π_in, T, ε̂,
      WithBodyRet(capability, Γ_outer, Ω_outer, π_outer, v)) —emit(ΔT, δε)→
      Returned(v, Ω', π', T ++ ΔT, append_effect(ε̂, δε))
```

Propagation conventions for scoped forms:

- `ObligBody(...)` and `WithBody(...)` are specification-only residual forms whose inner workflow
  steps by the same `A ⊢ κ —μ→ κ'` judgment.
- When the inner workflow rejects, that rejection propagates outward without being reclassified by the
  scope wrapper unless the owning SPEC-004 helper boundary explicitly says otherwise.
- When the inner workflow returns, the corresponding `...BodyRet(...)` residual form records the
  returned value so the unique exit rule can reconcile outgoing `Ω` / `π` state without discarding
  that terminal value.

### 7.4 Modal and Fallback Rules

Canonical rules in this group:

```text
MAYBE-PRIMARY | MAYBE-CAPTURE-REJECT | MAYBE-FALLBACK | MUST-STEP | MUST-REJECT
```

```text
(MAYBE-PRIMARY)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w_primary) —μ→ Running(Γ', Ω', π', T', ε̂', w_primary')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Maybe { primary = w_primary, fallback = w_fallback }) —μ→
      Running(Γ', Ω', π', T', ε̂',
        Maybe { primary = w_primary', fallback = w_fallback })

(MAYBE-CAPTURE-REJECT)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w_primary) —μ→ Rejected(err, Ω', π', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      Maybe { primary = w_primary, fallback = w_fallback }) —μ→
      Running(Γ, Ω, π, T, ε̂, MaybeReject(err, fallback = w_fallback))

(MAYBE-FALLBACK)
  fallback_permitted(err)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂,
      MaybeReject(err, fallback = w_fallback)) —silent→
      Running(Γ, Ω, π, T, ε̂, w_fallback)

(MUST-STEP)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w) —μ→ κ'
  κ' ≠ Rejected(_, _, _, _, _)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Must { workflow = w }) —μ→ wrap_must(κ')

(MUST-REJECT)
  A ⊢ Running(Γ, Ω, π, T, ε̂, w) —μ→ Rejected(err, Ω', π', T', ε̂')
  strengthen_must_rejection(err) ↝ err'
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Must { workflow = w }) —μ→
      Rejected(err', Ω', π', T', ε̂')
```

Side conditions:

- `MAYBE-FALLBACK` applies only for rejection classes that the canonical contract admits as
  fallback-triggering; all other rejections propagate unchanged.
- `MaybeReject(...)` is specification-only staging notation for “primary branch has just rejected,
  pending fallback classification.”
- `wrap_must(κ')` means: if `κ'` remains running, rebuild the residual `Must { ... }`; if `κ'` is a
  successful terminal configuration, preserve that terminal configuration.
- `strengthen_must_rejection(err)` is owned by the same mandatory-success boundary already defined by
  SPEC-004; this document does not flatten that contract into ad hoc new machine rules.

### 7.5 Receive and Concurrency Rules

Canonical rules in this group:

```text
RECEIVE-SELECTED | RECEIVE-FALLBACK | RECEIVE-FALLTHROUGH | RECEIVE-BLOCKED |
PAR-ENTER | PAR-STEP | PAR-AGGREGATE | PAR-REJECT
```

`Receive` delegates selection and classification to the already-owned helper boundary.

```text
(RECEIVE-SELECTED)
  select_receive_outcome(mode, control, arms, Γ, Ω, π)
    ↝ Selected(msg, ΔΓ, body, ΔΩ, Δπ, ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Receive { mode, arms, control }) —emit(ΔT, δε)→
      Running(Γ ⊕ ΔΓ, Ω ⊗ ΔΩ, π ⊗ Δπ, T ++ ΔT, append_effect(ε̂, δε), body)

(RECEIVE-FALLBACK)
  select_receive_outcome(mode, control, arms, Γ, Ω, π)
    ↝ Fallback(body, ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Receive { mode, arms, control }) —emit(ΔT, δε)→
      Running(Γ, Ω, π, T ++ ΔT, append_effect(ε̂, δε), body)

(RECEIVE-FALLTHROUGH)
  select_receive_outcome(mode, control, arms, Γ, Ω, π)
    ↝ Fallthrough(ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Receive { mode, arms, control }) —emit(ΔT, δε)→
      Returned(Null, Ω, π, T ++ ΔT, append_effect(ε̂, δε))
```

`RECEIVE-BLOCKED` is a classification rule, not a transition rule. It records when a receive is live
but presently non-progressing.

```text
(RECEIVE-BLOCKED)
  select_receive_outcome(mode, control, arms, Γ, Ω, π) ↝ Blocked
  ───────────────────────────────────────────────────────────────
  Running(Γ, Ω, π, T, ε̂, Receive { mode, arms, control }) is blocked/suspended
```

Side conditions:

- `Blocked` is admissible only for receive modes whose canonical contract permits waiting.
- Non-blocking miss remains `RECEIVE-FALLTHROUGH`, not `RECEIVE-BLOCKED`.
- Receive-side rejection remains helper-owned and reconstructs the existing SPEC-004 receive failure
  taxonomy rather than inventing a distinct small-step error channel.

`Par` remains interleaving-compatible and helper-backed. The canonical `Par { workflows }` form is
presented through specification-only residual `ParState(bs)` so branch-local progress can be named
explicitly without adding new surfaced syntax. The rule presentation below fixes the branch-local
step and terminal-shape contract without collapsing the semantics into left-to-right sequencing.

```text
(PAR-ENTER)
  initialize_parallel_branches(Γ, Ω, π, T, ε̂, workflows) ↝ bs
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Par { workflows }) —silent→ Running(Γ, Ω, π, T, ε̂, ParState(bs))

(PAR-STEP)
  i ∈ active_indices(bs)
  A ⊢ branch_config(bs, i) —μ→ branch_config'(bs, i)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, ParState(bs)) —μ→
      Running(Γ, Ω, π, T, ε̂, ParState(update_branch(bs, i, branch_config'(bs, i))))

(PAR-AGGREGATE)
  all_terminal(bs)
  combine_parallel_outcomes(Γ, Ω, π, T, ε̂, bs)
    ↝ ParallelReturn(vs, Ω', π', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, ParState(bs)) —silent→
      Returned(vs, Ω', π', T', ε̂')

(PAR-REJECT)
  all_terminal(bs)
  combine_parallel_outcomes(Γ, Ω, π, T, ε̂, bs)
    ↝ ParallelReject(err, Ω', π', T', ε̂')
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, ParState(bs)) —silent→
      Rejected(err, Ω', π', T', ε̂')
```

Side conditions:

- `ParState(bs)` is specification-only residual notation that makes branch-local progress explicit.
  It is not surfaced syntax and does not prescribe one machine layout.
- each branch entry in `bs` denotes one branch-local configuration carrying its own `Γi`, `Ωi`, `πi`,
  `Ti`, `ε̂i`, and eventual terminal payload.
- the parent carriers written on the enclosing `Running(...)` configuration during `PAR-STEP` are the
  pre-aggregation parent carriers, not a hidden shared mutable store for all branches.
- `PAR-STEP` permits any branch index admitted by the concurrency contract; presentation order here is
  not a scheduler commitment.
- branch terminal arrival is represented inside `ParState(bs)` branch entries rather than by a
  separate surfaced workflow form.
- `PAR-AGGREGATE` and `PAR-REJECT` are defined only when every branch entry is terminal; blocked or
  merely running branches keep the enclosing state in `ParState(bs)`.
- `combine_parallel_outcomes(...)` remains the authoritative owner of branch-result collation,
  cumulative-carrier aggregation, and concurrent rejection combination.
- list-valued success collation is by branch index, not by proof of execution order.
- No rule in this family imposes sequential short-circuiting inconsistent with the accepted helper-
  backed concurrent contract.

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

The closeout verdict from [TASK-426](../plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md)
still applies here: this specification is faithful and compatible, but current implementation
support remains partial for cumulative carriers, retained completion packaging, and fully explicit
helper-backed `Par` aggregation.

For the explicit row-by-row compatibility audit against [SPEC-004](SPEC-004-SEMANTICS.md) and
[MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md), see
[TASK-426](../plan/tasks/TASK-426-spec-025-big-step-and-runtime-compatibility-audit.md).

### 9.1 Receive Realization Evidence

Evidence class: direct for the observed non-blocking/fallback behavior; distributed/partial for the blocked-state carrier story.

Current inspected interpreter evidence supports the following receive-path correspondence claims:

- non-blocking `Receive` falls through observably when no arm matches,
- timeout or wildcard receive continues through the wildcard arm,
- blocking receive waits for message arrival rather than introducing semantic stuckness.

This is consistent with the canonical `RECEIVE-FALLTHROUGH`, `RECEIVE-BLOCKED`, and selected/fallback
rule families defined above.

### 9.2 Coarse Runtime Outcome State Evidence

Evidence class: partial/reconstructed.

Current runtime-side evidence supports a coarse correspondence classification among:

- active residual execution,
- blocked/suspended waiting,
- terminal outcome, and
- invalid/runtime-failure boundaries.

That coarse correspondence surface is not itself the small-step semantics, but it is now directly
packaged by the runtime as `RuntimeOutcomeState` after TASK-405. It remains coarser than the semantic
taxonomy in §5: in particular, semantic terminal rejection/failure and runtime-side invalid/terminated
control state are distinguished normatively here even where runtime packaging is intentionally more
conservative.

### 9.3 Control and Retained Completion Evidence

Evidence class: mixed — direct for control-authority lifecycle and retained completion observation,
partial for exact full `CompletionPayload` parity.

Current runtime evidence is now direct for the existence of runtime-owned control authority, sealed
retained completion records, and completion waiting/observation APIs, while remaining conservative
about exact parity with the full semantic `CompletionPayload` contract.

- Control-link lifecycle and terminal invalidation are directly evidenced as runtime-owned
  boundaries.
- Retained completion observation is directly evidenced through sealed retained records and a
  dedicated wait path, but those runtime carriers are still a conservative realization rather than a
  proof that every semantic completion dimension is packaged identically to `SPEC-004`.
- Current runtime support preserves honest retained slices for terminal result/effect/obligation/
  provenance observation, but full exact trace transport and one fully authoritative cumulative
  execution-record carrier remain outside current evidence.
- Accordingly, this specification preserves the `SPEC-004` completion/control contract normatively
  without overclaiming complete runtime parity for every terminal semantic dimension.

This is evidence for the correspondence boundary, not a replacement for the semantic carriers used in
this specification.

### 9.4 Current Parallel Realization Boundary

Evidence class: partial/reconstructed.

Current interpreter evidence shows `Par` being realized by concurrent branch execution followed by
aggregate result collection. This is partial implementation evidence that terminal aggregation remains
a distinct boundary, which is consistent with the helper-backed `PAR-AGGREGATE` / `PAR-REJECT`
stance defined here.

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

- fully mechanized meta-proofs over the rule set given here,
- stronger mechanized reuse lemmas and execution-record packaging above the helper/state-taxonomy
  contract already frozen here,
- concrete runtime machine mapping and queue/tombstone representation,
- full interpreter/runtime alignment closeout across all five layers.

These are owned by the downstream MCE-006 / MCE-007 alignment work rather than by this document.
