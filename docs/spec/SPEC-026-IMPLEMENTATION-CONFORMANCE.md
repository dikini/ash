# SPEC-026: Implementation Conformance Contract

## Status: Draft

## 1. Overview

This specification freezes the canonical implementation-conformance contract for Ash.

It exists so Rust, Lean, and future alternate Ash implementations can be judged against one explicit
contract instead of reconstructing conformance expectations indirectly from
[SPEC-004](SPEC-004-SEMANTICS.md), [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md),
[SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), and the surrounding planning or evidence
corpus.

This document is a conformance contract, not a replacement semantics document.

- [SPEC-004](SPEC-004-SEMANTICS.md) remains the normative owner of big-step / terminal workflow
  meaning.
- [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) remains the normative owner of the
  workflow-first small-step and state-taxonomy contract.
- [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) remains the normative owner of user-visible
  and tooling-visible runtime observations.
- This document defines how implementations are compared against those surfaces, what dimensions
  must be preserved at each surface, and where implementation freedom is allowed.

## 1.1 Scope

This specification defines:

1. the authoritative conformance surfaces for Ash implementations,
2. the preserved semantic dimensions required at each surface,
3. the permitted implementation freedom at each surface,
4. the bounded nondeterminism allowances for helper-owned concurrency, `receive`, and externally
   non-observable behavior,
5. the comparison rules used when exact step ordering is not required, and
6. the contract future differential-testing artifacts must target.

This specification does not:

- certify that any current implementation already satisfies every conformance surface,
- replace the underlying semantic or observable specifications,
- require one concrete abstract machine, runtime storage layout, scheduler, queue representation, or
  helper API shape,
- require one specific Rust carrier shape for cumulative semantic state, or
- promote informative runtime evidence into stronger semantic truth than the canonical specs state.

## 1.2 Normative vs Informative Material

Unless a section is explicitly marked informative, it is normative.

In particular:

- §§2-7 are normative.
- §8 is informative-only and records the current evidence boundary for the Rust runtime.

Informative implementation evidence may justify conservative wording, but it does not redefine the
canonical contract.

## 2. Authority Hierarchy

Ash conformance is judged by the following authority hierarchy:

1. canonical semantic and observable specifications;
2. this conformance contract, which freezes how those specifications are used to judge
   implementations;
3. source/handoff reference contracts for their own layer-specific boundaries;
4. planning artifacts, closeout notes, audits, and implementation evidence.

Normatively:

- code may realize the semantics, but code is not the canonical semantic source of truth;
- this specification does not move authority from the canonical specs into current Rust behavior;
- planning artifacts and MCE notes may supply design history or evidence, but they are not the final
  semantic authority for implementation conformance.

The primary conformance surfaces are summarized below.

| Surface | Normative owner(s) | Comparison object |
|---|---|---|
| big-step / terminal semantic conformance | [SPEC-004](SPEC-004-SEMANTICS.md) with canonical IR/value contracts from [SPEC-001](SPEC-001-IR.md) and [SPEC-020](SPEC-020-ADT-TYPES.md) where relevant | terminal workflow meaning |
| small-step / state-taxonomy conformance | [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) plus the helper-boundary contracts still owned by [SPEC-004](SPEC-004-SEMANTICS.md) | admissible configuration transitions, blocked/stuck distinctions, and terminal reconstruction |
| runtime-observable conformance | [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md), plus [SPEC-005](SPEC-005-CLI.md) and [SPEC-011](SPEC-011-REPL.md) where those documents define a surfaced boundary | user/tool-visible outcomes |

An implementation may be assessed separately on each surface. Stronger conformance on one surface
must not be assumed automatically from weaker evidence on another.

## 3. Big-Step / Terminal Semantic Conformance

### 3.1 Authoritative Sources

Big-step / terminal semantic conformance is judged primarily against:

- [SPEC-004](SPEC-004-SEMANTICS.md),
- [SPEC-001](SPEC-001-IR.md) for canonical workflow and expression forms,
- [SPEC-020](SPEC-020-ADT-TYPES.md) where constructor/pattern behavior affects runtime meaning.

If these documents disagree with implementation behavior, the documents win.

### 3.2 Required Preserved Dimensions

An implementation conforms on the big-step surface only if, for every canonical input within the
claimed coverage boundary, it preserves the terminal semantic dimensions required by
[SPEC-004](SPEC-004-SEMANTICS.md):

1. terminal outcome class: `Return(...)` versus `Reject(...)`;
2. returned value or rejection-owning error category, including constructor identity and payload
   meaning where relevant;
3. terminal effect classification;
4. terminal trace meaning and execution-order constraints where the spec fixes them;
5. terminal obligation state `Ω'`;
6. terminal provenance state `π'`.

If the canonical big-step semantics determines one unique terminal outcome, the implementation must
produce that same semantic outcome. If the canonical semantics admits a bounded set of terminal
outcomes, the implementation must produce one member of that set and no outcome outside it.

### 3.3 Permitted Implementation Freedom

Big-step conformance does not require:

- one concrete evaluator structure,
- one recursion strategy,
- one helper decomposition,
- one runtime memory layout,
- one internal trace-storage format, or
- one specific representation of environments, obligations, provenance, or effects in code.

Implementations may differ internally so long as the terminal semantic projection remains within the
admitted big-step contract.

### 3.4 Bounded Nondeterminism on the Big-Step Surface

Big-step nondeterminism is allowed only where the underlying canonical semantics already allows it.
This includes helper-owned concurrency and helper-owned selection boundaries such as `receive`
outcomes that are intentionally not reduced to one implementation-specific scheduling story.

Big-step nondeterminism is bounded as follows:

1. an implementation may choose any terminal outcome admitted by the authoritative helper contract;
2. it may not invent new terminal outcomes not admitted by the canonical semantics;
3. it may not convert a semantically blocked/waiting condition into an unrelated rejection unless the
   canonical owner of that boundary allows such rejection;
4. it may not collapse helper-owned concurrency into a different sequential contract when the specs
   preserve concurrent combination instead.

## 4. Small-Step / State-Taxonomy Conformance

### 4.1 Authoritative Sources

Small-step conformance is judged primarily against:

- [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md),
- the helper-boundary and terminal-reconstruction contracts in
  [SPEC-004](SPEC-004-SEMANTICS.md).

The accepted MCE notes may explain provenance or current runtime evidence, but they are not the
normative owner of small-step conformance.

### 4.2 Required Preserved Dimensions

An implementation conforms on the small-step surface only if it preserves the following semantic
dimensions for the executions it claims to realize:

1. the workflow-first subject of reduction over canonical workflow configurations;
2. the canonical configuration taxonomy, including running versus terminal configurations;
3. the split between cumulative state carried in configurations and local deltas carried in labels;
4. the distinction between blocked/suspended states and semantic stuckness;
5. the ownership boundary for helper-managed operations such as receive selection, `Par`
   aggregation, and control/completion observation;
6. terminal reconstruction back to the [SPEC-004](SPEC-004-SEMANTICS.md) outcome contract.

For this surface, conformance is not only about the final terminal result. It is also about whether
intermediate states and step classes remain within the admitted state taxonomy.

### 4.3 Permitted Implementation Freedom

Small-step conformance does not require:

- one concrete abstract machine,
- one scheduler,
- one queue layout,
- one branch-state carrier shape,
- one exact count of silent/internal steps,
- one exact helper API naming scheme, or
- one requirement that helper-owned atomic regions be exposed as user-visible machine steps.

An implementation may expand, collapse, or repackage internal execution so long as the resulting
stepwise behavior still admits a faithful mapping to the canonical small-step contract.

### 4.4 Bounded Nondeterminism on the Small-Step Surface

Small-step nondeterminism is bounded, not arbitrary.

Allowed nondeterminism includes only:

1. interleavings of semantically concurrent `Par` progress that remain compatible with the canonical
   helper-owned aggregation contract;
2. helper-owned `receive` choices among messages, timeout continuation, wildcard continuation, or
   fallthrough cases already admitted by the authoritative receive contract;
3. implementation-local silent-step segmentation that does not change the admitted state-taxonomy or
   terminal reconstruction.

Disallowed nondeterminism includes:

- changing blocked/suspended behavior into semantic stuckness,
- imposing a fake left-to-right sequential `Par` contract where the canonical semantics preserves a
  concurrent one,
- exposing implementation accidents as new semantic step classes,
- changing the owning boundary of a runtime failure, pattern failure, or terminal-control outcome.

## 5. Runtime-Observable Conformance

### 5.1 Authoritative Sources

Runtime-observable conformance is judged primarily against:

- [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md),
- [SPEC-005](SPEC-005-CLI.md) for CLI-owned process/tooling boundaries,
- [SPEC-011](SPEC-011-REPL.md) for REPL-owned interactive boundaries.

Where a visible runtime outcome is derived from internal semantics, [SPEC-004](SPEC-004-SEMANTICS.md)
and [SPEC-025](SPEC-025-SMALL-STEP-OPERATIONAL-SEMANTICS.md) still determine the meaning being
projected, but [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) owns the user/tool-visible form.

### 5.2 Required Preserved Dimensions

An implementation conforms on the runtime-observable surface only if it preserves the required
observable dimensions of the relevant interface, including where applicable:

1. success, warning, denial, and error category distinctions that the observable contract keeps
   separate;
2. `ash run` process-exit behavior and exit-code derivation;
3. visible ADT/value display shape where the specs fix it;
4. visible CLI/REPL error-class distinctions;
5. visible control/instance/monitor role distinctions where the specs require them.

Observable conformance is judged only on surfaced behavior. It does not require one internal runtime
representation.

### 5.3 Permitted Implementation Freedom

Observable conformance allows variation in:

- internal storage and execution strategy,
- hidden supervision mechanics,
- non-observable scheduling,
- punctuation or incidental formatting where the observable specs do not freeze one exact string,
- descendant management after the externally observable process exit where
  [SPEC-021](SPEC-021-RUNTIME-OBSERVABLE-BEHAVIOR.md) explicitly leaves it implementation-defined.

### 5.4 Bounded Nondeterminism on the Observable Surface

Observable nondeterminism is allowed only where the observable contract leaves behavior open.

Examples:

1. after `main` exits, descendant fate is not an observable conformance target for `ash run`;
2. when formatting punctuation is not frozen, implementations may differ while preserving the same
   visible semantic distinctions;
3. non-observable internal completion bookkeeping does not become part of observable conformance
   merely because a runtime happens to expose it internally.

Observable nondeterminism does not allow:

- changing required exit-code derivation,
- collapsing required visible error categories into one undifferentiated outcome,
- leaking implementation-defined internal field names or internal tags as if they were canonical
  user-visible syntax.

## 6. Cross-Surface Comparison Rules

### 6.1 Surface-Specific Comparison Objects

Different conformance surfaces compare different projections of the same program.

- big-step conformance compares terminal semantic outcomes;
- small-step conformance compares admissible state-transition behavior and terminal reconstruction;
- runtime-observable conformance compares surfaced observations only.

A harness or proof obligation must therefore declare which surface it is testing. A failure on one
surface does not automatically imply failure on all others, although some failures may propagate.

### 6.2 Comparison When Exact Step Ordering Is Not Required

When the canonical contract does not require one exact internal ordering, implementations are
compared by admissible equivalence class rather than by raw step-for-step identity.

The normative comparison rules are:

1. silent/internal stutter may vary if the same admitted semantic milestones are preserved;
2. helper-owned atomic regions may be collapsed or expanded if the same owning boundary and terminal
   projection are preserved;
3. `Par` executions are compared up to admissible interleaving of independent branch progress rather
   than one implementation-specific schedule;
4. receive behavior is compared by admitted selection/fallback/timeout outcome class and resulting
   continuation, not by queue-probe count, poll frequency, or one concrete mailbox algorithm;
5. when a surface is deterministic, the allowed equivalence class has size one.

### 6.3 `Par`-Specific Rule

For helper-owned concurrency, implementations need not expose the same branch-step order.
Conformance requires only that:

1. each realized branch behavior is individually admitted by the canonical semantics,
2. the aggregate branch outcome is admitted by the canonical helper-owned combination contract,
3. no implementation rewrites the meaning into a contradictory sequential contract,
4. observable results stay within the allowed set for the chosen surface.

### 6.4 `Receive`-Specific Rule

For `receive`, implementations need not expose the same internal queue-inspection sequence.
Conformance requires only that:

1. the selected message, wildcard branch, timeout branch, fallthrough, or blocking behavior is one
   allowed by the authoritative receive contract,
2. the resulting continuation preserves the owning failure/selection boundary,
3. external tests do not overassert on unowned mailbox internals.

## 7. Differential Testing and Result-Format Contract

Future differential-testing artifacts must be parameterized by this specification.

Normatively, any canonical result format or harness produced later for conformance testing must:

1. declare the conformance surface being tested;
2. identify the authoritative source documents for that test case;
3. record whether the expected result is deterministic or set-valued under bounded nondeterminism;
4. encode only the projection required by the chosen surface;
5. treat implementation-defined or explicitly non-observable dimensions as out of scope rather than
   as failure conditions.

Accordingly:

- [TASK-438](../plan/tasks/TASK-438-canonical-ir-semantics-corpus-and-result-format.md) must define
  corpus/result artifacts against the surface definitions frozen here;
- [TASK-439](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md) must judge the
  Rust runtime against the allowed result set for each declared surface rather than against one
  accidental reference ordering;
- future Lean/reference work must consume the same surface definitions instead of inventing a second
  comparison contract.

A differential harness fails an implementation only when the implementation's output falls outside
the allowed set for the declared surface.

## 8. Informative: Current Rust Runtime Evidence Boundary

This section is informative only.

The current Rust runtime must not be described as already satisfying every stronger conformance
surface in full.

The current evidence packet from
[MCE-006](../ideas/minimal-core/MCE-006-SMALL-STEP-IR.md) and
[MCE-007](../ideas/minimal-core/MCE-007-FULL-ALIGNMENT.md) supports a conservative status summary:

| Surface | Current conservative status |
|---|---|
| big-step / terminal semantic conformance | strongest current target, with substantial direct implementation evidence for many workflow outcomes, but not a blanket proof or certification of full corpus closure |
| small-step / state-taxonomy conformance | partial / indirect; current runtime evidence remains incomplete for authoritative cumulative `π`, `T`, and `ε̂` carriers, uniform blocked-versus-suspended packaging, full retained completion parity, and fully explicit helper-backed `Par` aggregation |
| runtime-observable conformance | partially realized and strongest where visible exit behavior, visible value shape, and surfaced error/output boundaries are already specified and tested, but still not evidence of full small-step carrier closure |

Therefore:

1. the target conformance contract is stronger than the currently evidenced Rust runtime in some
   areas;
2. observable or coarse terminal alignment must not be misreported as full small-step conformance;
3. partial runtime evidence is still useful, but it remains informative evidence rather than the
   canonical source of truth.

## 9. Implementation Tasks

- [TASK-428](../plan/tasks/TASK-428-implementation-conformance-contract.md): create this contract
- [TASK-438](../plan/tasks/TASK-438-canonical-ir-semantics-corpus-and-result-format.md): define the
  canonical corpus/result format against this contract
- [TASK-439](../plan/tasks/TASK-439-differential-conformance-harness-rust-first.md): implement the
  first harness against this contract
- [TASK-440](../plan/tasks/TASK-440-lean-reference-refresh-plan-against-current-semantic-corpus.md):
  align Lean/reference planning with this contract

## 10. Changelog

### 2026-04-07

- Initial implementation-conformance contract published.
- Froze the three canonical conformance surfaces: big-step, small-step, and runtime-observable.
- Froze bounded nondeterminism and differential-testing comparison rules without overclaiming current
  runtime closure.
