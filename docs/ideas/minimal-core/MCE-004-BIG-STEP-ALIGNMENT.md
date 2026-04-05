---
status: accepted
created: 2026-03-30
last-revised: 2026-04-05
related-plan-tasks: [TASK-393]
tags: [semantics, big-step, surface, ir, alignment]
---

# MCE-004: Big-Step Semantics Alignment

## Status Summary

This exploration is resolved as documentation/spec alignment work.

The main big-step alignment questions it originally raised are now answered by the current corpus:

- `SPEC-001` defines the canonical IR contract.
- The parser-to-core lowering contract defines how surface syntax lowers into that canonical IR.
- `SPEC-004`, especially after [TASK-350](../../plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md), defines the proof-shaped big-step meaning of the canonical IR forms.
- [MCE-002 Audit Report](MCE-002-IR-AUDIT-REPORT.md) resolved the relevant IR-form questions needed for this alignment pass.

## Scope

In scope:

- surface syntax → canonical IR lowering
- canonical IR → big-step semantics
- documenting the resolved contract between those layers

Out of scope:

- small-step semantics
- interpreter or runtime implementation internals
- optimization or lowering implementation strategy beyond the normative handoff

## Resolved Alignment

### 1. Surface syntax lowers to canonical IR first

The big-step semantics are defined over canonical IR, not over raw surface syntax.

That relationship is now explicit across the docs set:

- `docs/spec/SPEC-001-IR.md` defines the canonical workflow, expression, and pattern forms.
- `docs/reference/parser-to-core-lowering-contract.md` defines the surface-to-core lowering boundary.
- `docs/spec/SPEC-004-SEMANTICS.md` states that its rules define the meaning of the canonical core IR from `SPEC-001` and treat surface constructs as semantically relevant only through lowering.

So the intended evaluation pipeline is:

1. parse surface syntax
2. lower to canonical IR
3. evaluate the canonical IR under the big-step semantics

### 2. `Seq` remains a primitive workflow form

This question is resolved.

Per [MCE-002 Audit Report](MCE-002-IR-AUDIT-REPORT.md), `Workflow::Seq` must remain primitive:

- `Seq` composes workflows;
- `Let` binds an `Expr` and continues with a workflow;
- therefore there is no valid `Seq` → `Let` rewrite in the current IR contract.

This aligns with `SPEC-001`, which keeps `Seq` in the canonical workflow set, and with `SPEC-004`, which gives workflow forms their own explicit big-step treatment rather than trying to erase sequencing into expression binding.

### 3. `Par` uses branch-effect join plus helper-backed aggregation

This question is resolved in `SPEC-004`.

Parallel composition is not left as an unspecified effect merge. The current big-step semantics define concurrent combination using helper-backed contracts, with the all-success case combining branch effects by join:

- terminal effect = `⊔ eff_i` across successful branches;
- concurrent obligation/provenance aggregation is handled by the explicit helper contracts around parallel outcome combination.

So MCE-004 no longer needs to ask whether parallel effects join; the normative answer is yes, with helper-owned aggregation contracts for the concurrent outcome details.

### 4. Spawned completion seals the child's own authoritative terminal state

This question is also resolved in `SPEC-004`.

The `CompletionPayload` for a spawned child seals the child's terminal result together with the child's authoritative:

- obligation state,
- provenance state,
- effect summary.

That means spawned completion is modeled as the completion of the child workflow instance itself, not as mutation of one shared ambient completion payload. The payload records the child's terminal obligation state as its own completion artifact.

### 5. `Match` remains a primitive core expression; `if let` is sugar

This question is resolved jointly by `MCE-002`, `SPEC-001`, and the lowering contract.

- `Expr::Match` remains a primitive canonical core expression.
- Surface `match` lowers directly to `Expr::Match`.
- Surface `if let` lowers to `Expr::Match` with a wildcard fallback arm.

So the core semantics need direct `Match` behavior, and `if let` does not require its own canonical runtime form.

### 6. Big-step semantics already have the explicit judgment structure MCE-004 asked for

The original exploration asked for clearer workflow/expression/pattern judgments and more explicit helper contracts.

That work is now present in `SPEC-004` after [TASK-350](../../plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md):

- explicit workflow big-step judgment
- explicit expression evaluation judgment
- explicit pattern matching judgment
- explicit helper-relation contracts
- centralized propagation and failure conventions

This means MCE-004 has effectively been promoted from an exploratory gap list into a resolved alignment note.

## Canonical Cross-References

- Canonical IR: [docs/spec/SPEC-001-IR.md](../../spec/SPEC-001-IR.md)
- Big-step semantics: [docs/spec/SPEC-004-SEMANTICS.md](../../spec/SPEC-004-SEMANTICS.md)
- Surface-to-core lowering: [docs/reference/parser-to-core-lowering-contract.md](../../reference/parser-to-core-lowering-contract.md)
- IR audit: [MCE-002 Audit Report](MCE-002-IR-AUDIT-REPORT.md)
- Proof-shaped semantics completion: [TASK-350](../../plan/tasks/TASK-350-revise-spec-004-to-complete-big-step-core-semantics.md)
- Planning closeout: [TASK-393](../../plan/tasks/TASK-393-big-step-semantics-alignment.md)

## Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-30 | Exploration created | Initial alignment questions captured while semantics and IR contracts were still being normalized |
| 2026-04-05 | Exploration accepted as resolved planning artifact | The current corpus already settles the Seq, Par, spawn-completion, Match, and judgment-structure questions |
