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
  evaluate_guard(C, P, action, guard, Γ, Ω, π) ↝ GuardOk
  perform_action(C, action, provenance, Γ, Ω, π) ↝ ActOk(v, Ω', π', ΔT, δε)
  ───────────────────────────────────────────────────────────────
  A ⊢ Running(Γ, Ω, π, T, ε̂, Act { action, guard, provenance }) —emit(ΔT, δε)→
      Returned(v, Ω', π', T ++ ΔT, append_effect(ε̂, δε))
```

Failure-side conditions for this family:

- `observe_capability(...)`, `form_proposal(...)`, `apply_policy(...)`, `check_obligation(...)`,
  `evaluate_guard(...)`, and `perform_action(...)` may fail only through their already-owned SPEC-004
  rejection boundaries.
- Policy denial remains owned by `DECIDE-STEP` and reconstructs the existing
  `PolicyViolation(policy, v)` category.
- Obligation failure remains owned by `CHECK-STEP` and reconstructs the existing
  `ObligationViolation(obligation)` category.
- Guard failure remains owned by `ACT-STEP` and reconstructs the existing
  `GuardViolation(action, guard)` category.
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
- `PAR-STEP` permits any branch index admitted by the concurrency contract; presentation order here is
  not a scheduler commitment.
- branch terminal arrival is represented inside `ParState(bs)` branch entries rather than by a
  separate surfaced workflow form.
- `combine_parallel_outcomes(...)` remains the authoritative owner of branch-result collation,
  cumulative-carrier aggregation, and concurrent rejection combination.
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
- standalone proof-oriented helper-contract packages and state-taxonomy closure beyond the rule layer,
- concrete runtime machine mapping and queue/tombstone representation,
- full interpreter/runtime alignment closeout across all five layers.

These are owned by the downstream MCE-006 / MCE-007 alignment work rather than by this document.