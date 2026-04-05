---
status: closeout-published
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-394, TASK-395, TASK-396, TASK-397, TASK-398, TASK-399, TASK-400]
related-plan-notes: TASK-397 is reconciled complete as the earlier framing/scaffold task whose intended outputs were materially realized by the published matrix, residual-gap classification layer, and closeout/signoff contract in this document.
tags: [alignment, surface, ir, semantics, interpreter, consolidation]
---

# MCE-007: Full Layer Alignment

## Problem Statement

All layers of Ash must remain consistent:

surface syntax → canonical IR → big-step semantics → small-step semantics → interpreter/runtime.

This exploration is the full-stack consolidation point. It now treats:

- [MCE-004](MCE-004-BIG-STEP-ALIGNMENT.md) as resolved for the surface → IR → big-step side; and
- [MCE-005](MCE-005-SMALL-STEP.md) as materially defined for the canonical small-step backbone.

The heaviest remaining dependency is therefore no longer “what is the small-step backbone?” but “how is that backbone realized by the interpreter/runtime?” — the central concern of [MCE-006](MCE-006-SMALL-STEP-IR.md), whose Phase 63 closeout now provides a frozen runtime-evidence packet for downstream ingestion.

## Scope

In scope:

- all five layers: Surface, IR, Big-step, Small-step, Interpreter;
- cross-layer consistency checks;
- documentation of correspondence obligations between adjacent layers;
- identification of remaining drift after MCE-004 and Phase 61.

Out of scope:

- new feature design;
- optimization work;
- low-level runtime implementation changes.

Related but separate:

- [MCE-004](MCE-004-BIG-STEP-ALIGNMENT.md): accepted prerequisite for surface → IR → big-step
- [MCE-005](MCE-005-SMALL-STEP.md): accepted small-step planning/design backbone
- [MCE-006](MCE-006-SMALL-STEP-IR.md): frozen runtime/interpreter evidence packet already consumed into the MCE-007 matrix

## The Five Layers

```text
┌─────────────────────────────────────┐
│ Layer 1: Surface Syntax             │ User-facing Ash language
└──────────────┬──────────────────────┘
               │ lowering
               ▼
┌─────────────────────────────────────┐
│ Layer 2: Canonical IR               │ `SPEC-001`
└──────────────┬──────────────────────┘
               │ big-step meaning
               ▼
┌─────────────────────────────────────┐
│ Layer 3: Big-Step Semantics         │ `SPEC-004`
└──────────────┬──────────────────────┘
               │ small-step refinement
               ▼
┌─────────────────────────────────────┐
│ Layer 4: Small-Step Semantics       │ MCE-005 backbone
└──────────────┬──────────────────────┘
               │ runtime realization
               ▼
┌─────────────────────────────────────┐
│ Layer 5: Interpreter / Runtime      │ executable implementation
└─────────────────────────────────────┘
```

## Alignment Verification Obligations

For each canonical construct, MCE-007 ultimately needs:

1. Surface → IR: lowering contract defined.
2. IR → Big-step: big-step semantic rule defined.
3. Big-step → Small-step: correspondence argument defined.
4. Small-step → Interpreter: runtime realization argument defined.

## Current Alignment State

### Resolved inputs

The following are already fixed and should not be reopened here:

- `Workflow::Seq` remains primitive.
- `Expr::Match` remains primitive, and `if let` lowers to `Expr::Match`.
- `Par` big-step aggregation is helper-backed with successful branch-effect join.
- Spawn completion seals the child workflow's own authoritative terminal state in `CompletionPayload`.
- Small-step is workflow-first, uses the accepted configuration/label split from MCE-005, keeps expressions/patterns atomic in v1, and distinguishes blocked/suspended states from stuckness.

### Updated verification matrix

| Construct family | Surface→IR | IR→Big | Big→Small | Small→Interp | Status |
|---|---|---|---|---|---|
| Sequencing / binding / branching | ✅ | ✅ | Backbone fixed; per-form correspondence still to package | partial / follow-up required — ordinary residual execution is directly realized by recursive execution over residual `Workflow` ASTs with direct `Γ` carriage and distributed `A = (C, P)` holders, but terminal runtime results still do not package authoritative cumulative `Ω` / `π` / `T` / `ε̂` state. | partial / follow-up required |
| Pattern-driven control | ✅ | ✅ | Backbone fixed; correspondence still to package | partial / follow-up required — match/pattern/guard-driven control follows the ordinary recursive workflow path and uses the existing pattern/guard failure boundary in `Err(ExecError)`, but rejected-vs-runtime-failure subclasses are still multiplexed through the broad non-success channel rather than uniformly separated. | partial / follow-up required |
| Receive / blocking behavior | ✅ | ✅ | Backbone fixed; blocked-vs-stuck settled | partial / follow-up required — blocking receive is real but distributed across mailbox/stream wait loops, while explicit suspension uses `YieldState` plus `ExecError::YieldSuspended`. Follow-up remains for one authoritative blocked-vs-terminal-vs-invalid runtime class. | partial / follow-up required |
| Parallel composition | ✅ | ✅ | Backbone fixed; interleaving + helper aggregation settled | partial / follow-up required — `Workflow::Par` is realized as bulk async child execution with cloned branch-local `Context` state, shared runtime registries, and direct all-success `Value::List(...)` collation. Follow-up remains for explicit branch-step/interleaving realization and helper-backed cumulative-state aggregation over `Ω` / `π` / `T` / `ε̂`. | partial / follow-up required |
| Capability / policy / obligation workflows | ✅ | ✅ | Backbone fixed; detailed correspondence still to package | partial / follow-up required — ambient capability/policy context is distributed across runtime holders and policy checks are evidenced at decision/action/receive boundaries; obligation state is genuinely carried in `Context` / `RoleContext`. Follow-up remains because authoritative terminal `Ω` packaging and cumulative `π` / `T` / `ε̂` carriers are still partial or missing. | partial / follow-up required |
| Spawn / completion observation contracts | ✅ | ✅ | Helper/runtime contract fixed as input | partial / follow-up required — spawn-time control authority and later pause/resume/kill/check-health boundaries are directly realized through `ControlLinkRegistry`. Follow-up remains because retained completion-payload-style observation and completion-wait carriers are still weak/missing on the inspected main path. | partial / follow-up required |

The remaining `❓` weight is now classified row-by-row. It is concentrated in runtime/interpreter realization limits around blocked-state unification, cumulative semantic carriers, retained completion payloads, and helper-backed concurrent aggregation — not in the existence of a small-step backbone.

### Runtime ingestion notes from the frozen MCE-006 Phase 63 packet

- Workflow-configuration → runtime-structure mapping: the interpreter is now frozen as a hybrid control realization. Ordinary progress is direct recursive execution over residual `Workflow` ASTs, while effective machine state is partly externalized into runtime-owned registries for mailbox/control/proxy/suspension behavior.
- Blocked/suspended-state realization: blocking `Receive` is operationally real but mostly implicit/distributed through mailbox/stream wait loops; explicit proxy/yield suspension is carried by stored `YieldState` continuations plus `ExecError::YieldSuspended`.
- `Par` realization: current runtime evidence supports concurrent child-future execution, cloned branch-local `Context` state, shared mailbox/control/proxy/suspension infrastructure, and direct successful value collation into `Value::List(...)`; it does not justify claiming full small-step-style branch scheduler or helper-backed cumulative-state aggregation.
- Observable-preservation verdict: the current interpreter partially realizes the accepted MCE-005 backbone for observable purposes. Successful return class is direct, non-success is visible but multiplexed, `Ω` is only reconstructed/approximated, and authoritative runtime carriers for `π`, `T`, and `ε̂` remain weak/missing.
- Spawn/completion authority and retained-completion limits: control authority lifecycle is directly realized and stable after spawn registration, but retained `CompletionPayload`-style observation is still not evidenced as one equally explicit runtime carrier.

## Residual-Gap Classification Layer

TASK-398 populated the row-level Small-step → Interpreter column. TASK-399 adds the residual-gap layer that says whether each remaining partial row is an accepted owner-bound limitation or a true five-layer drift item that still needs follow-on work.

### Residual-gap categories

| Category | Meaning | Drift? | Typical owner |
|---|---|---|---|
| `closed` | No remaining cross-layer action is needed for the row/issue. | no | none |
| `packaging-only` | Alignment is materially settled; remaining work is to package correspondence or closeout prose more clearly. | no | MCE-007 closeout / TASK-400 |
| `accepted partiality` | The corpus already accepts the current limitation as an owned follow-up boundary; the row stays partial, but the limitation is not treated as present five-layer drift. | no | later runtime cleanup or narrow taxonomy follow-up |
| `true residual drift` | A real cross-layer tension remains between the accepted semantic story and the current runtime realization; later work must resolve it rather than merely describe it. | yes | future runtime/interpreter follow-on, with TASK-400 carrying the closeout checklist |

### What is already accepted and not drift

The following points should not be reopened as residual drift claims:

- ordinary residual workflow execution via recursive execution over residual `Workflow` ASTs;
- distributed runtime holders for `A = (C, P)` and direct `Γ` carriage;
- control-link authority lifecycle through `ControlLinkRegistry`;
- the accepted upstream semantic choices from MCE-004 and MCE-005;
- the need to package the big-step ↔ small-step correspondence more readably for closeout.

### Row-level residual classification

Interpret the row labels below as residual-summary labels, not as permission to flatten away mixed cases in final closeout. TASK-400 should preserve any row that contains both locally aligned evidence and a distinct unresolved drift dependency.

| Construct family | Residual classification | Accepted partiality vs true drift | Owner | TASK-400 handoff |
|---|---|---|---|---|
| Sequencing / binding / branching | `mixed: accepted partiality + true residual drift` | Direct residual execution itself is accepted as aligned, but the lack of authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging is still a true residual drift item that affects this row through the terminal observables it should preserve. | future runtime/interpreter follow-on for carrier packaging; TASK-400 for closeout wording | keep row open and explicitly note the split between accepted local execution alignment and unresolved cumulative-carrier drift |
| Pattern-driven control | `accepted partiality` | Match/pattern/guard control is directly realized enough for current alignment purposes. The remaining open item is subtype precision inside the broad non-success channel: rejected-vs-runtime-failure separation is still weak. | later runtime taxonomy cleanup | record as owned limitation, not as a reopened semantics conflict |
| Receive / blocking behavior | `true residual drift` | Blocking is operationally real, but one authoritative blocked-vs-terminal-vs-invalid runtime class is still missing. That remains a genuine cross-layer tension with the accepted state taxonomy. | future runtime/interpreter follow-on | treat as required follow-up in the closeout checklist |
| Parallel composition | `true residual drift` | Concurrent child execution is real, but the current runtime does not yet justify the accepted helper-backed cumulative-state aggregation / explicit interleaving story over `Ω` / `π` / `T` / `ε̂`. | future runtime/interpreter follow-on | keep `Par` called out explicitly in closeout and drift-prevention material |
| Capability / policy / obligation workflows | `true residual drift` | Runtime policy/obligation hooks exist, but authoritative cumulative `Ω` packaging and cumulative `π` / `T` / `ε̂` carriers are still partial or missing. That is more than packaging: the accepted semantic carriers are not yet uniformly realized. | future runtime/interpreter follow-on | preserve as an open carrier-realization item |
| Spawn / completion observation contracts | `true residual drift` | Spawn/control authority is directly realized, but retained completion-payload-style observation remains weak/missing on the inspected main path. That is a direct mismatch against the accepted completion-observation contract. | future runtime/interpreter follow-on | keep separate from generic carrier work so completion observation does not disappear into broader cleanup |

### Residual issue register

This is the frozen TASK-399 handoff table for downstream closeout work.

| Residual issue | Category | Why this classification is conservative | Owner | Affected rows |
|---|---|---|---|---|
| Packaged big-step ↔ small-step correspondence | `packaging-only` | The accepted backbone already exists. The remaining work is explanatory closeout packaging, not new semantic or runtime design. | MCE-007 closeout / TASK-400 | all rows indirectly |
| Rejected-vs-runtime-failure subtype separation | `accepted partiality` | The runtime already exposes a broad non-success boundary. Finer subtype separation is still desirable, but the corpus currently treats this as a taxonomy/cleanup issue rather than a present contradiction in the accepted control story. | later runtime taxonomy cleanup | pattern-driven control |
| One authoritative blocked / terminal / invalid runtime class | `true residual drift` | The accepted semantics rely on a clearer state-class boundary than the current mixed async-wait / `ExecError` realization provides. | future runtime/interpreter follow-on | receive / blocking behavior |
| Authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging | `true residual drift` | The current runtime only partially reconstructs or distributes these carriers. Because the accepted five-layer story treats them as cumulative semantic state, the gap is not merely editorial. | future runtime/interpreter follow-on | sequencing; capability / policy / obligation workflows; indirectly `Par` |
| Retained completion-payload-style observation | `true residual drift` | Upstream semantics already accept authoritative completion payload observation, but the inspected runtime path does not yet expose an equally explicit retained carrier. | future runtime/interpreter follow-on | spawn / completion observation contracts |
| Full helper-backed concurrent cumulative-state aggregation for `Par` | `true residual drift` | `join_all(...)` plus `Value::List(...)` is useful evidence of concurrency, but it is not yet the accepted semantic aggregation contract. | future runtime/interpreter follow-on | parallel composition |

### Accepted partiality vs true residual drift summary

Accepted partiality / owner-bound limitations:

- packaged big-step ↔ small-step correspondence remains a closeout-packaging task owned by MCE-007 / TASK-400;
- rejected-vs-runtime-failure subtype separation remains a later runtime taxonomy cleanup item;
- direct recursive residual execution, distributed ambient holders, and control-link authority realization are accepted evidence, not drift.

Mixed case to keep explicit:

- sequencing / binding / branching contains both accepted local execution alignment and one true residual drift dependency through missing authoritative cumulative-carrier packaging.

True residual drift requiring follow-on work:

- one authoritative blocked / terminal / invalid runtime class;
- authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging;
- retained completion-payload-style observation;
- full helper-backed concurrent cumulative-state aggregation for `Par`.

## Remaining Full-Stack Gaps

### Gap 1: Packaged big-step ↔ small-step correspondence

After Phase 61, the backbone is no longer the blocker. The remaining semantic packaging work is to make the correspondence explicit enough that later readers can see how:

- terminal outcomes are reconstructed from terminal configurations;
- traces/effects/provenance/obligations are preserved across repeated steps;
- blocked/suspended states refine, rather than contradict, the `SPEC-004` worldview.

This is the closeout space tracked by Phase 61 correspondence work and the relevant sections of MCE-005.

### Gap 2: Small-step ↔ interpreter correspondence residuals

This is no longer an un-ingested placeholder gap. TASK-398 has now consumed the frozen MCE-006 handoff packet into the matrix above.

What remains open is the explicitly classified residual subset. The tables above distinguish accepted partiality from true drift; not every item below is itself a fresh drift claim.

- one authoritative blocked / terminal / invalid runtime class;
- authoritative cumulative runtime carriers for `π`, `T`, and `ε̂`;
- stronger terminal `Ω` packaging;
- retained completion-payload-style observation;
- full helper-backed concurrent cumulative-state aggregation for `Par`.

### Gap 3: Ongoing drift prevention

This documentation/closeout gap is now closed by TASK-400. The drift-prevention checklist in the final closeout section below is the durable guardrail for future changes. It does not erase the remaining true residual drift items; it freezes how later changes must review them instead of letting them disappear into incidental edits.

## Deliverables

MCE-007 now provides:

1. a construct-by-construct five-layer alignment matrix;
2. explicit correspondence notes for the remaining nontrivial constructs;
3. a runtime-realization summary consuming the frozen MCE-006 packet, including row-level Small-step → Interpreter classifications;
4. a drift-prevention checklist for future language/runtime changes;
5. one explicit residual-gap register distinguishing packaging-only work, accepted partiality, and true residual drift with owners.

## Final Closeout, Signoff, and Drift-Prevention Checklist

TASK-400 closes MCE-007 as a documentation/planning/signoff artifact, not as a claim that all five-layer runtime drift is gone. The accepted corpus state is now frozen enough that a reader can inspect this document alone and answer four questions:

- what is aligned across the five layers today;
- what remains an accepted owner-bound limitation rather than present drift;
- what is still true residual drift requiring later runtime/interpreter follow-on;
- what future edits must re-check so the corpus does not drift again.

### Accepted five-layer matrix state at closeout

| Construct family | Accepted five-layer state at closeout | Residual-gap state now frozen in closeout |
|---|---|---|
| Sequencing / binding / branching | Surface → IR and IR → Big-step are accepted from MCE-004. Big-step → Small-step uses the accepted MCE-005 backbone and packaged correspondence. Small-step → Interpreter has accepted local execution evidence: ordinary residual execution over recursive `Workflow` ASTs, direct `Γ` carriage, and distributed ambient holders for `A = (C, P)`. | Mixed case preserved intentionally: local execution alignment is accepted, but authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging is still true residual drift. This row is therefore closeout-stable but not fully runtime-closed. |
| Pattern-driven control | Surface lowering, big-step meaning, and small-step control shape are accepted. Runtime evidence is good enough for current closeout purposes: match/pattern/guard-driven control follows the ordinary workflow path and honors the broad non-success boundary. | `accepted partiality`: rejected-vs-runtime-failure subtype precision is still weak, but the corpus currently treats that as later runtime taxonomy cleanup rather than present five-layer contradiction. |
| Receive / blocking behavior | Surface, IR, big-step, and small-step blocked-vs-stuck framing are accepted. Runtime evidence confirms that blocking receive is operationally real and that explicit suspension uses `YieldState` plus `ExecError::YieldSuspended`. | `true residual drift`: one authoritative blocked-vs-terminal-vs-invalid runtime class is still missing, so the accepted state taxonomy is not yet uniformly realized by the runtime. |
| Parallel composition | Surface, IR, big-step, and small-step `Par` structure are accepted, including helper-backed all-success aggregation as the semantic target. Runtime evidence confirms concurrent child execution, cloned branch-local `Context`, shared runtime registries, and direct successful `Value::List(...)` collation. | `true residual drift`: the runtime still does not justify claiming full helper-backed cumulative-state aggregation over `Ω` / `π` / `T` / `ε̂`, nor an equally explicit interleaving/branch-step realization. |
| Capability / policy / obligation workflows | Surface contracts, IR carriers, big-step meaning, and small-step backbone are accepted. Runtime evidence shows ambient capability/policy context at decision/action/receive boundaries and genuine obligation carriage in `Context` / `RoleContext`. | `true residual drift`: authoritative cumulative `Ω` packaging and cumulative `π` / `T` / `ε̂` carriers remain partial or missing, so the accepted semantic carriers are not yet uniformly realized at runtime. |
| Spawn / completion observation contracts | Surface, IR, big-step, and small-step inputs are accepted, including the upstream completion-observation contract. Runtime evidence confirms spawn-time control authority and pause/resume/kill/check-health boundaries through `ControlLinkRegistry`. | `true residual drift`: retained completion-payload-style observation remains weak or missing on the inspected main runtime path, so this row stays open independently of the broader cumulative-carrier work. |

### Current residual-gap state after TASK-400 closeout

The previous `packaging-only` item is now consumed by this closeout section itself:

- packaged big-step ↔ small-step correspondence: closed as closeout packaging, not left as a new follow-up.

The residual set that remains after documentation closeout is:

- `accepted partiality`: rejected-vs-runtime-failure subtype separation for pattern-driven control remains an owner-bound runtime taxonomy cleanup item.
- `mixed: accepted partiality + true residual drift`: sequencing / binding / branching remains explicitly mixed; accepted local execution evidence does not cancel the unresolved cumulative-carrier drift dependency.
- `true residual drift`: one authoritative blocked / terminal / invalid runtime class.
- `true residual drift`: authoritative cumulative `Ω` / `π` / `T` / `ε̂` packaging.
- `true residual drift`: retained completion-payload-style observation.
- `true residual drift`: full helper-backed concurrent cumulative-state aggregation for `Par`.

### Signoff conditions

#### Closed enough for MCE-007 documentation/signoff

MCE-007 is signed off as complete documentation/planning/closeout work when all of the following are true:

1. every construct-family row has a published five-layer state and an explicit residual classification;
2. mixed rows are preserved explicitly rather than flattened into a single optimistic status label;
3. the accepted upstream MCE-004 and MCE-005 decisions remain frozen and are not reopened here;
4. the residual register names owners for every non-closed item;
5. the closeout text distinguishes accepted partiality from true residual drift;
6. the corpus does not claim that runtime/interpreter realization is fully closed while the frozen true residual drift set still exists;
7. a durable future-change checklist is published in the same artifact.

Those conditions are now met.

#### What would count as fully five-layer closed, rather than merely closeout-complete

The full five-layer stack should only be called materially closed if later runtime/interpreter work resolves or legitimately re-specifies the current true residual drift set, namely:

1. one authoritative blocked-vs-terminal-vs-invalid runtime class exists and matches the accepted state taxonomy;
2. authoritative cumulative `Ω` / `π` / `T` / `ε̂` carriers are packaged/preserved across ordinary execution, capability/policy/obligation paths, and `Par` outcomes;
3. retained completion-payload-style observation is exposed with strength comparable to the accepted completion contract;
4. `Par` either realizes the accepted helper-backed aggregation/interleaving story or the accepted semantic target is revised explicitly upstream;
5. any future closure claim still preserves the sequencing / binding / branching row's mixed-case nuance until the cumulative-carrier drift dependency is actually resolved.

#### Still open / follow-up required

Alignment should still be treated as open follow-up work, not fully closed, whenever any of the following is true:

- the true residual drift set above remains unresolved;
- a future edit changes canonical constructs, lowering, semantics, or runtime behavior without updating the affected MCE-007 rows and residual register;
- a change collapses a mixed row into a single simplified label and thereby hides a remaining drift dependency;
- a new runtime claim is made without evidence that adjacent-layer observables still line up.

### Future-change drift-prevention checklist

When any canonical construct family, runtime realization, or observable contract changes, review all applicable items below before calling the change aligned:

1. Surface lowering and user-facing docs
   - Re-check parser/lowering docs and any surface-sugar notes.
   - Confirm that canonical construct families did not gain or lose hidden lowering nuance.
   - If sequencing / binding / branching changes, explicitly ask whether local execution alignment changed, whether cumulative-carrier behavior changed, or both.

2. `SPEC-001` canonical IR
   - Confirm the canonical IR carrier and construct inventory still match the matrix row being cited.
   - Re-check whether the change alters residual workflow shape, terminal result carriers, or `Par` structure.

3. `SPEC-004` big-step semantics
   - Re-check workflow/expression/pattern/helper contracts.
   - Re-check blocked, terminal, invalid, completion, and helper-backed aggregation language.
   - Do not claim runtime closure if the big-step observable contract changed but runtime carriers were not re-audited.

4. MCE-005 small-step backbone
   - Re-check configuration/label responsibilities, blocked-vs-stuck taxonomy, and accepted workflow rule inventory.
   - If a change touches control, suspension, or `Par`, verify that the accepted small-step target still matches the runtime claim.

5. MCE-006 runtime/interpreter correspondence packet
   - Re-check the semantic-carrier → runtime mapping.
   - Re-check blocked/suspended realization, `Par` realization, observable-preservation notes, and completion/control authority language.
   - Do not rely on stale Phase 63 wording if runtime structures changed materially.

6. Interpreter/runtime-facing docs and implementation-adjacent notes
   - Re-check any docs describing runtime holders, registries, completion payloads, mailbox wait behavior, yield suspension, and branch aggregation.
   - Ask explicitly whether the change improves, preserves, or worsens the frozen true residual drift set.

7. MCE-007 matrix rows and residual register
   - Update the affected row state, residual classification, owner, and signoff wording.
   - Preserve mixed rows explicitly; do not flatten sequencing / binding / branching while cumulative-carrier drift remains.
   - If a prior true drift item is resolved, record what evidence now closes it.
   - If a new drift item appears, add it with an owner rather than letting it hide inside prose.

8. Closeout/signoff language
   - Re-check whether the change is only documentation packaging, accepted partiality, or true residual drift.
   - Only use “fully closed” if the true residual drift conditions above are actually satisfied.
   - Otherwise keep the stronger but narrower claim: “MCE-007 closeout/signoff complete, with named residual drift still open.”

## Dependencies

This exploration depends on:

- MCE-002 (IR inventory known)
- MCE-004 (surface → IR → big-step alignment accepted)
- MCE-005 (small-step backbone accepted in Phase 61)
- MCE-006 (frozen runtime/interpreter evidence packet now available for TASK-398 ingestion)

Recommendation: treat MCE-007 as having a published closeout/signoff artifact, while keeping the true residual runtime-side drift set explicitly open. The remaining work is runtime/interpreter follow-on for those residual items, not additional Phase 63 theory or missing MCE-007 packaging.

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Need a complete alignment view |
| 2026-04-05 | Reframed after Phase 61 | MCE-005 is now materially defined; remaining dependency weight is on MCE-006 and runtime/interpreter correspondence |
| 2026-04-05 | Ingested the frozen Phase 63 runtime-evidence packet into the Small-step → Interpreter matrix | TASK-398 converts the prior placeholder column into row-level classifications. All rows remain conservatively partial / follow-up required to some degree, with stronger direct evidence for ordinary residual execution and pattern-driven control than for blocking, `Par`, capability/obligation observables, and spawn/completion retention |
| 2026-04-05 | Classified residual gaps into owner-bound partiality versus true residual drift | TASK-399 adds a dedicated residual-gap layer so later closeout work can distinguish packaging-only work and accepted runtime limitations from real unresolved cross-layer tensions around blocked-state classification, cumulative carriers, retained completion observation, and `Par` aggregation |
| 2026-04-05 | Published final MCE-007 closeout/signoff conditions and drift-prevention checklist | TASK-400 freezes the accepted five-layer matrix state, consumes the prior packaging-only closeout work, keeps sequencing / binding / branching explicitly mixed, and distinguishes “documentation closeout complete” from the still-open true residual drift set |
