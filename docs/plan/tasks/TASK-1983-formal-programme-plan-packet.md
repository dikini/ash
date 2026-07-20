# TASK-1983: Formal Programme Plan Packet

**Status:** Complete
**Phase:** PLAN-202: Formal Semantics And Verification Programme

## Description

Create the formal programme deliverable that establishes the documentation and semantic authority
needed before Ash adopts Verus for Rust implementation proofs or designs an Ash-native
`spec`/`proof` system.

The programme must turn the current documentation corpus into an explicit, machine-navigable
authority graph; plan the quarantine and archival of displaced documentation; sequence semantic
cleanup and deprecated-code removal; bound a formal CPS calculus; define end-to-end traceability;
and specify the first two Verus proof pilots.

## Requirements

- Define the authoritative corpus and conflict-resolution order for grammar, typing, lowering,
  Core/CPS, runtime semantics, observable behavior, and implementation conformance.
- Define a future-proof canonical documentation structure optimized for human, LLM, and agent
  retrieval without treating generated context packs as authority.
- Define a reversible archival and supersession plan for documentation displaced by the canonical
  corpus.
- Convert documentation findings into an evidence-led deprecation/removal workstream that extends,
  rather than duplicates, Phase 201.
- Bound the formal CPS calculus, including its theorem targets, explicit exclusions, and
  relationship to surface and runtime semantics.
- Define a stable traceability schema connecting requirements, grammar productions, semantic rules,
  Core/CPS forms, Rust implementation symbols, tests, proof obligations, and evidence artifacts.
- Specify two ordered Verus proof pilots with theorem statements, trusted boundaries, eligible Rust
  targets, acceptance gates, and stop/go criteria.
- Define gates between documentation canonicalization, calculus formalization, Rust verification,
  and later Ash-native proof-system design/implementation.
- Add the programme to `docs/plan/PLAN-INDEX.md`, update `CHANGELOG.md`, and preserve the current
  orientation-index contract.

## TDD Steps

1. Inventory existing authority, formalization, documentation-gate, CPS, Lean, and semantic-cleanup
   artifacts before drafting the programme.
2. Write the programme so every required workstream has inputs, outputs, dependencies, acceptance
   evidence, and failure/stop conditions.
3. Review the programme independently for documentation-authority gaps and formal-verification
   feasibility.
4. Integrate review findings and verify that all referenced files and task links resolve.
5. Run the orientation-index self-test and documentation gate.

## Completion Checklist

- [x] PLAN-202 defines all seven requested programme components.
- [x] The canonical corpus and authority hierarchy are explicit.
- [x] Documentation migration and archival are reversible and gated.
- [x] Deprecated-code removal is audit-led and reconciled with Phase 201.
- [x] The CPS calculus scope, theorem ladder, and exclusions are explicit.
- [x] The traceability schema has stable identifiers and validation rules.
- [x] Two Verus pilots have concrete targets, theorems, TCBs, and acceptance criteria.
- [x] Parallel-track dependencies and stop/go gates are explicit.
- [x] PLAN-INDEX and CHANGELOG are updated.
- [x] Documentation validation passes.

## Evidence

The plan packet adds PLAN-202 and TASK-1983 through TASK-1994, indexes Phase 202, and records the
change under `[Unreleased]`.

Two read-only specialist audits covered documentation governance/archival and CPS/Verus feasibility.
Their findings caused the programme to:

- extend SPEC-071/DESIGN-035/DESIGN-042 through a manifest overlay;
- record the workflow-first versus target-semantics authority conflicts;
- stage the calculus as `λAsh-CPS₀` followed by `λAsh-Effect`;
- select Core row algebra and frame-ordered dispatch as the first two pilots; and
- require an isolated toolchain spike because neither `verus` nor `cargo-verus` is currently
  installed or configured in the workspace.

An independent review identified and then confirmed remediation of stop/no-go, dependency,
metadata-schema, pilot-scope, TCB, repository-scope, and Phase 201 handoff issues.

Validation:

```text
python3 tools/docs/validate_orientation_indexes.py --self-test
orientation-index-check: OK

bash scripts/check-docs-gate.sh
docs-gate: markdown links checked=1464 missing=0
orientation-index-check: OK
docs-gate: OK
```
