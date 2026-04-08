# Semantic Execution Record Contract

## Status

TASK-432 reference contract.

## Purpose

This reference freezes the canonical runtime-facing semantic execution-record contract for Ash.

It exists to make one previously diffuse boundary explicit:

- [SPEC-004: Operational Semantics](../spec/SPEC-004-SEMANTICS.md) already fixes the authoritative terminal semantic dimensions;
- [SPEC-025: Small-Step Operational Semantics](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) already fixes the workflow-first configuration carriers and terminal projection boundary;
- [MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md) and [MCE-007](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md) already record that current runtime realization remains partial for cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- this document now freezes the runtime-facing contract that later implementation work must target so those carriers do not keep drifting into ad hoc packaging.

This is a semantic/reference contract, not a concrete Rust API or storage design.

## Scope

This contract defines:

1. the canonical semantic execution record for one workflow execution instance;
2. the semantic meaning of its cumulative carrier fields;
3. the allowed runtime-facing phase/state classification carried with those fields;
4. the exact terminal projection from that record back to `SPEC-004` workflow outcomes and completion-style payloads;
5. the exact-vs-conservative boundary for staged runtime adoption.

This contract does not define:

- one required Rust struct, trait, enum, or public API shape;
- one required scheduler, queue, tombstone, or registry layout;
- one required representation of `Γ` or the residual workflow term;
- the full `Par` branch-local aggregation contract, which remains follow-on work;
- full retained-completion API parity, transport shape, or external exposure policy.

## 1. Contract Role

The semantic execution record is the canonical runtime-facing package for cumulative semantic state of one authoritative execution instance.

Typical instances include:

- one top-level workflow execution;
- one spawned child workflow execution;
- one branch-local execution instance if a runtime realizes `Par` branches as distinct runtime-owned executions.

An implementation may realize the record directly, indirectly, or distributively across several internal holders, but if it claims to expose or preserve an execution record for semantic conformance, the record must reconstruct the same semantic content frozen here.

The execution record is not the same thing as:

- the full `SPEC-025` configuration vocabulary, which also includes `Γ` and residual workflow shape;
- a retained-completion record used for later observation after terminal sealing;
- a per-step label stream carrying only local `ΔT` / `δε` deltas;
- a scheduler or abstract-machine frame layout.

## 2. Canonical Semantic Execution Record

The canonical semantic execution record has the following semantic shape:

```text
ER ::= ExecRecord {
  phase: ρ,
  obligations: Ω,
  provenance: π,
  trace: T,
  effects: ε̂,
}
```

with phase/state carrier:

```text
ρ ::= Running
    | Blocked(β)
    | Terminal(Return(v))
    | Terminal(Reject(err))
    | Invalid(ι)
```

where:

- `Ω` is the current cumulative obligation state;
- `π` is the current cumulative provenance state;
- `T` is the cumulative execution trace prefix in execution order;
- `ε̂` is the cumulative effect-summary carrier whose terminal projection reconstructs the `SPEC-004` terminal effect and `CompletionPayload.effects` view;
- `v` is the terminal returned value;
- `err` is the terminal rejection/error owned by the `SPEC-004` failure taxonomy;
- `β` classifies an admitted blocked/suspended wait boundary;
- `ι` classifies an invalid or inadmissible runtime state that is outside the admitted semantic execution path.

This contract intentionally does not require `Γ` or the residual workflow term to be embedded in the runtime-facing execution record. Those remain part of the `SPEC-025` semantic configuration contract, but their concrete runtime packaging is intentionally left implementation-neutral here.

### 2.1 Phase Classification Requirements

The minimum admitted blocked/wait classification is:

```text
β ::= ReceiveWait
    | CompletionObservationWait
    | ControlWait
    | HelperWait(name)
```

An implementation may refine these with more detail, but it must not collapse the following distinctions:

1. `Running` is progress-capable now.
2. `Blocked(β)` is nonterminal waiting, not rejection and not semantic stuckness.
3. `Terminal(Return(v))` is terminal success.
4. `Terminal(Reject(err))` is terminal rejection/failure already owned by the ordinary `SPEC-004` runtime failure boundary.
5. `Invalid(ι)` is outside the admitted semantic execution path and must not be used as a catch-all replacement for ordinary workflow rejection.

In particular:

- receive waiting must remain classifiable as blocked/suspended rather than silently reclassified as rejection;
- completion-observation waiting must remain classifiable as blocked/suspended rather than flattened into active polling or terminal completion;
- ordinary policy, obligation, guard, pattern, capability, mailbox, or runtime failures that the semantics already classify as `Reject(err, ...)` belong in `Terminal(Reject(err))`, not in `Invalid(ι)`.

### 2.2 Cumulative-Carriage Laws

The execution record is cumulative, not delta-shaped.

Normatively:

1. `Ω`, `π`, `T`, and `ε̂` are the authoritative cumulative state accumulated so far for that execution instance.
2. `T` is ordered in execution order and is not a bag or set.
3. `ε̂` is the cumulative effect-summary state, not merely the most recent step effect.
4. `Blocked(β)` preserves the last committed cumulative `Ω`, `π`, `T`, and `ε̂`; blocked waiting does not erase or downgrade previously accumulated semantic state.
5. Once an execution record is terminal or invalid, its semantic contents are sealed for that execution instance; later retained-completion or control-observation surfaces may project from that sealed state, but do not retroactively rewrite it.

## 3. Relationship to `SPEC-025` Configurations

This contract is the runtime-facing packaging counterpart to the `SPEC-025` carrier vocabulary.

The intended semantic correspondence is:

```text
Running(Γ, Ω, π, T, ε̂, w)
  ↦ ExecRecord { phase = Running or Blocked(β), obligations = Ω, provenance = π, trace = T, effects = ε̂ }

Returned(v, Ω, π, T, ε̂)
  ↦ ExecRecord { phase = Terminal(Return(v)), obligations = Ω, provenance = π, trace = T, effects = ε̂ }

Rejected(err, Ω, π, T, ε̂)
  ↦ ExecRecord { phase = Terminal(Reject(err)), obligations = Ω, provenance = π, trace = T, effects = ε̂ }
```

The `Running` versus `Blocked(β)` refinement is runtime-facing: `SPEC-025` keeps blocked/suspended ownership in the rule/taxonomy/helper contract, while this execution-record contract makes that distinction explicitly packageable for runtime-facing use.

`Invalid(ι)` has no ordinary `SPEC-004` workflow-outcome analogue. It exists only to classify non-admitted runtime states or invalid execution carriers. It must not swallow ordinary semantic rejection cases that should instead remain `Terminal(Reject(err))`.

## 4. Terminal Projection Contract

Terminal projection is defined only for records whose phase is `Terminal(...)`.

### 4.1 Workflow Outcome Projection

The canonical projection back to `SPEC-004` workflow outcomes is:

```text
project_workflow(
  ExecRecord { phase = Terminal(Return(v)), obligations = Ω, provenance = π, trace = T, effects = ε̂ }
) = Return(v, terminal_effect(ε̂), T, Ω, π)

project_workflow(
  ExecRecord { phase = Terminal(Reject(err)), obligations = Ω, provenance = π, trace = T, effects = ε̂ }
) = Reject(err, terminal_effect(ε̂), T, Ω, π)
```

This projection reconstructs exactly the `SPEC-004` workflow outcome dimensions:

| `SPEC-004` dimension | Source in execution record |
|---|---|
| returned value / rejection error | `phase = Terminal(Return(v))` or `phase = Terminal(Reject(err))` |
| terminal effect | `terminal_effect(ε̂)` |
| cumulative trace | `T` |
| terminal obligation state | `Ω` |
| terminal provenance | `π` |

No other runtime-facing carrier may claim to be the canonical semantic execution record if it cannot support this projection exactly.

### 4.2 Completion-Payload Projection

When a terminal execution record is observed through a spawned-child completion boundary, the corresponding semantic completion-style payload projection is:

```text
project_completion(
  ExecRecord { phase = Terminal(Return(v)), obligations = Ω, provenance = π, effects = ε̂, ... }
) = {
  result: Ok(v),
  obligations: Ω,
  provenance: π,
  effects: effect_trace(ε̂),
}

project_completion(
  ExecRecord { phase = Terminal(Reject(err)), obligations = Ω, provenance = π, effects = ε̂, ... }
) = {
  result: Err(err),
  obligations: Ω,
  provenance: π,
  effects: effect_trace(ε̂),
}
```

where:

```text
effect_trace(ε̂) = EffectTrace {
  terminal: terminal_effect(ε̂),
  reached: reached_effects(ε̂),
}
```

Important boundary:

- `T` remains part of the authoritative semantic execution record and workflow-outcome projection;
- `T` is not part of `CompletionPayload` and therefore is not required to be transported by completion-style payload projection itself.

### 4.3 Undefined Projections for Nonterminal or Invalid Records

For:

- `phase = Running`,
- `phase = Blocked(β)`, or
- `phase = Invalid(ι)`,

`project_workflow(...)` and `project_completion(...)` are not defined.

A runtime may expose snapshots or classifications for those states, but it must not misrepresent them as authoritative terminal semantic outcomes.

## 5. Exactness Contract: Exact vs Conservative vs Out of Scope

The table below freezes the adoption boundary.

| Dimension | Exact for semantic conformance | Conservative / staged runtime adoption allowed | Out of scope here |
|---|---|---|---|
| Terminal success vs rejection class | Yes. `Return` vs `Reject` must project exactly from terminal record phase. | Coarser helper/runtime classification may exist on auxiliary surfaces, but not as a substitute for the canonical record. | Concrete enum/API names. |
| Blocked/suspended vs terminal vs invalid distinction | Yes. The record must preserve the frozen taxonomy and not collapse blocked waiting into rejection or success. | TASK-405-style coarse classes may remain as auxiliary projections while full record substrate is being adopted. | Scheduler internals and polling strategy. |
| Blocked wait-boundary family | Yes at the family level relevant to semantic ownership: receive wait, completion observation wait, control wait, or other helper wait. | Finer runtime subkinds may remain implementation-defined or partially surfaced. | One universal machine wait object shape. |
| `Ω` in the execution record | Yes. Current and terminal `Ω` carried by the canonical record must be exact. | Retained-completion surfaces may temporarily expose narrower honest subsets, such as TASK-410's terminal-visible obligations slice, if clearly labeled conservative/partial. | Exact retained API packaging. |
| `π` in the execution record | Yes. Current and terminal `π` carried by the canonical record must be exact. | Retained-completion surfaces may temporarily expose conservative lineage/identity slices, such as TASK-411's retained provenance summary, if clearly labeled conservative/partial. | Concrete provenance storage layout. |
| `T` in the execution record | Yes. The canonical record must carry exact cumulative trace in execution order. | External retained-completion or runtime-observable surfaces may omit `T`; omission there does not redefine the canonical record. | Trace compression/export format. |
| `ε̂` terminal effect projection | Yes. `terminal_effect(ε̂)` must reconstruct the exact `SPEC-004` terminal effect. | Auxiliary retained surfaces may expose conservative upper bounds while full parity remains open, but those are not the exact execution-record projection. | Internal effect-summary algorithm details, if semantically equivalent. |
| `ε̂` reached-effect projection | Yes if claiming full completion-style payload projection from the canonical record. | Conservative reached-effect upper bounds remain acceptable on staged retained-completion surfaces such as TASK-409's current summary, provided they are not mislabeled exact. | One required summary data structure. |
| Terminal result payload | Yes. `v` or `err` must be exact in the terminal record. | Separate retained records may transport the same payload indirectly or lazily. | Serialization/transport shape. |
| `Γ` and residual workflow `w` packaging | No. They remain owned by `SPEC-025` semantics, but this runtime-facing record contract does not require them to be part of the canonical package. | Implementations may surface them privately or diagnostically. | One required runtime exposure. |
| `Par` branch-local aggregation details | No. Later work freezes that contract separately. | Current runtimes may remain partial here without violating this document. | Full branch-state / aggregation design. |
| Concrete retained-completion API parity | No. This contract only fixes what exact projection means, not the full external retained API. | TASK-406 through TASK-412 style staged slices remain allowed if honestly labeled. | Final retained-completion public API design. |

## 6. Helper-Boundary and State-Taxonomy Compatibility

This contract must be read together with the frozen helper/state-taxonomy contract from [TASK-430](../plan/tasks/TASK-430-small-step-helper-contracts-and-state-taxonomy.md) and the normative state-taxonomy sections of [SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md).

Normatively:

1. The execution record does not micro-step helper internals.
2. Helper-owned receive selection, completion observation, control observation, obligation/provenance transitions, and `Par` aggregation remain helper-owned boundaries.
3. The execution record packages the semantic state before and after those boundaries and, for blocked cases, the fact that the execution is waiting at one of those admitted boundaries.
4. `Blocked(CompletionObservationWait)` is a first-class admitted nonterminal class. It is not a rejection, not ordinary active execution, and not proof that the child lacks a terminal record; it means the observing execution is waiting on a helper/runtime-owned completion boundary.
5. `Blocked(ReceiveWait)` is likewise admitted nonterminal waiting rather than semantic stuckness.
6. `Invalid(ι)` is reserved for inadmissible runtime states, malformed carrier states, or implementation-side configuration errors that sit outside the admitted semantic path. It must not absorb helper-owned rejections that belong in `Terminal(Reject(err))`.

## 7. Relationship to TASK-405 Through TASK-412

TASK-405 through TASK-412 introduced real runtime-facing surfaces, but they do not by themselves constitute full execution-record closure.

### 7.1 TASK-405: Runtime Outcome/State Classification

[TASK-405](../plan/tasks/TASK-405-authoritative-runtime-outcome-state-classification.md) adds a conservative authoritative runtime classification surface.

Relationship to this contract:

- TASK-405's outcome/state class is a coarse projection of `phase`.
- It is compatible with this contract.
- It is not sufficient by itself to satisfy this contract because it does not carry exact cumulative `Ω`, `π`, `T`, and `ε̂`.

### 7.2 TASK-406 Through TASK-412: Retained Completion Observation

[TASK-406](../plan/tasks/TASK-406-retained-completion-payload-observation.md) through [TASK-412](../plan/tasks/TASK-412-dedicated-completion-wait-carrier.md) add retained completion observation, retained result/effect/obligation/provenance slices, and a wait surface.

Relationship to this contract:

- those surfaces are downstream terminal observations, not the canonical live execution record;
- they should be understood as projections from, or staged precursors toward, a sealed terminal execution record;
- their current conservative payload slices remain honest and useful, but they are not yet full semantic execution-record parity.

More specifically:

- TASK-408 retained `result` corresponds to the terminal `Ok(v)` / `Err(err)` projection;
- TASK-409 retained effects introduced the first conservative `effect_trace(ε̂)` slice, and TASK-437 now upgrades child-owned retained completions to exact `effect_trace(ε̂)` projection from the sealed terminal execution record while leaving control tombstones effect-payload-free;
- TASK-410 retained obligations correspond only to a terminal-visible obligations slice, not yet full exact `Ω` transport;
- TASK-411 retained provenance corresponds only to a conservative runtime-owned provenance slice, not yet full exact `π` transport;
- TASK-412 completion waiting is a wait surface over the retained terminal observation carrier, not a live execution-record wait handle exposing cumulative carriers in flight.

### 7.3 Honest Closure Boundary

Accordingly, the relationship to TASK-405 through TASK-412 is:

- compatible and intentionally staged,
- useful as runtime-facing precursor work,
- not yet full execution-record closure,
- not sufficient grounds to claim exact current runtime parity for cumulative `Ω` / `π` / `T` / `ε̂` carriage.

## 8. Implementation-Neutral Conformance Guidance

A runtime, proof artifact, or alternate implementation conforms to this contract when:

1. it preserves the semantic meaning of `phase`, `Ω`, `π`, `T`, and `ε̂` for one execution instance;
2. its terminal execution record projects exactly to the `SPEC-004` workflow outcome;
3. any claimed completion-style projection from the canonical record reconstructs `result`, `obligations`, `provenance`, and `effects` as defined above;
4. any weaker or conservative runtime surface is explicitly marked as weaker or conservative rather than silently presented as the canonical record.

Implementation freedom remains for:

- carrier storage layout;
- whether the record is materialized eagerly or reconstructed from sealed internal state;
- whether the runtime exposes the record publicly, internally, or only through a conformance/differential-testing harness;
- how runtime-private environment, residual control state, or scheduler data are represented.

## 9. Relationship to the Retained Completion Parity Contract

[Retained Completion Parity Contract](retained-completion-parity-contract.md) is the companion
reference for the retained terminal observation surface.

Relationship summary:

1. this execution-record contract remains the broader runtime-facing semantic carrier contract;
2. retained completion parity concerns only the semantic `CompletionPayload` dimensions,
   not the full execution-record state;
3. exact trace `T` remains owned by this execution-record contract and is intentionally not required
   for retained `CompletionPayload` parity;
4. weaker retained surfaces from TASK-406 through TASK-412 should be classified using the retained
   parity contract's exact / conservative / subset-only categories rather than described ad hoc.

## 10. Follow-On Boundary

This contract is the fixed input for the next runtime and conformance tasks, especially:

- [TASK-433: `ash-interp` Execution-Record Substrate](../plan/tasks/TASK-433-ash-interp-execution-record-substrate.md)
- [TASK-434: `Par` Branch-State and Aggregation Contract](../plan/tasks/TASK-434-par-branch-state-and-aggregation-contract.md)
- [TASK-436: Completion-Payload Parity Contract](../plan/tasks/TASK-436-completion-payload-parity-contract.md)
- [TASK-439: Differential Conformance Harness (Rust First)](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md)

Those tasks should implement or test against this contract directly rather than re-deciding:

- what the runtime-facing cumulative semantic carriers are;
- what exact terminal projection means;
- which current retained-completion surfaces are already exact versus only conservative;
- how blocked/completion-observation boundaries fit into the execution-record story.

## 11. TASK-434 Compatibility Note for `Par` Branch Execution Records

TASK-434 freezes the normative `Par` branch-state and aggregation contract in
[SPEC-025](../spec/SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md). This reference remains compatible with
that contract as follows.

### 10.1 Branch-Local Execution Record Meaning

If a runtime realizes `Par` branches as distinct runtime-owned execution instances, then each branch may
carry its own branch-local execution record:

```text
ExecRecord {
  phase: Running | Blocked(β) | Terminal(Return(v)) | Terminal(Reject(err)) | Invalid(ι),
  obligations: Ωi,
  provenance: πi,
  trace: Ti,
  effects: ε̂i,
}
```

Normatively, such a branch-local record denotes the same branch-local semantic carriers frozen by the
`Par` contract: branch-local `Ω`, `π`, `T`, `ε̂`, and eventual terminal payload for one branch execution
instance.

This does not require a runtime to expose each branch record publicly. It only fixes the semantic
meaning if the runtime claims to preserve or reconstruct branch-local execution records for conformance.

### 10.2 Aggregate Record Boundary

This reference still treats the canonical execution record as describing one authoritative execution
instance. For an enclosing `Par`, the aggregate execution record is the parent/enclosing record, not the
unordered bag of branch-local records.

Accordingly:

1. live branch-local records may coexist before aggregate completion;
2. blocked/suspended branch-local records remain nonterminal and are not collapsed into parent terminal
   completion;
3. the enclosing aggregate record becomes terminal only when the `Par` aggregation precondition holds,
   i.e. when the branch-state contract admits helper-backed aggregation of terminal branch outcomes;
4. if an implementation projects one enclosing terminal execution record for `Par`, the projected
   `Ω`, `π`, `T`, and `ε̂` must equal the helper-backed aggregate carriers admitted by the frozen `Par`
   contract, not a scheduler-accidental or first-observed approximation.

### 10.3 Conformance with Different Branch Orders

Different runtimes may realize or observe branch records in different orders and still conform, provided
that:

1. each branch record preserves exact branch-local carrier meaning for that branch;
2. blocked versus terminal versus invalid branch classification is preserved;
3. the enclosing terminal record, when one exists, projects exactly to the allowed aggregate `SPEC-004`
   outcome;
4. any variation is only in admitted branch interleaving or helper-owned aggregation latitude, not in
   loss of branch-local semantic content.
