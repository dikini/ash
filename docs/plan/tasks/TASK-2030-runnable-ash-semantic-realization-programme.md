# TASK-2030: Runnable Ash Semantic Realization Programme

**Status:** Complete — PLAN-203, TASK-2031, and TASK-2032 now assign the semantic,
shared-execution, client-parity, and runnability-matrix handoffs; canonical/agent reading paths
and the non-blocking Verus ledger agree with that programme. The calculus, corpus, orientation,
traceability, documentation, and whitespace validation gates pass.
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** [PLAN-203](../PLAN-203-RUNNABLE-ASH-SEMANTIC-REALIZATION.md)
**Depends on:** TASK-2029, TASK-2004, and TASK-2014

## Description

Align the planning and specification workflow around the selected production architecture:
Surface Ash lowers to Core, Core lowers to CPS, and one Engine-owned CPS executor realizes
admitted programs for both CLI and daemon clients. The λAsh calculi are the mathematical account
of CPS transitions and their Rust refinement; they are not additional lowering or execution paths.

## Requirements

- Record PLAN-203 as the execution-realization programme without changing the ownership of
  existing Surface, Core, CPS, admission, runtime, terminal, or conformance tasks.
- Correct the formal-semantics reading path so `λAsh-CPS₀` and its future `λAsh-Effect` extension
  explain CPS rather than receiving source lowering directly.
- Require PLAN-203 participating tasks to declare their run-route impact and their separately
  owned integration responsibility.
- Establish CLI/daemon parity as comparison of independently issued local Engine requests for the
  same source contract and normalized terminal result, not as a second evaluator or a
  legacy-runtime comparison.
- Keep Verus experimental and non-blocking. The semantic traceability graph records selected pilot
  evidence and deferred obligations; no unproved obligation may be represented as verified.

## Handoffs

- **Run-route impact:** `none`. This task changes no source, Core, CPS, admission, executor, or
  client behavior; it supplies the programme controls that future realization tasks use.
- **Consumes:** TASK-2029's layer/domain ownership policy; TASK-2004/TASK-2014's closed-admission
  execution architecture; the Canonical Core's lowering, operational, observable, and conformance
  rules.
- **Produces:** PLAN-203, the agent-facing executable-realization workflow, and the first planned
  [λAsh-Effect correspondence work item](TASK-2031-lambda-ash-effect-correspondence.md).
- **Downstream owner:** TASK-2031 owns the complete effect-calculus correspondence definition;
  TASK-2032 owns connection of completed handoffs to an active shared-Engine route and its client
  parity evidence.
- **Does not own:** implementation of a generic CPS executor, daemon transport, or the complete
  λAsh-Effect calculus. Those are follow-on realization tasks.
- **Integration/proof responsibility:** PLAN-203 owns the composition gates. Individual feature
  tasks continue to own only their declared layer/domain; Verus pilots are recorded in
  traceability and do not block execution delivery.

## TDD and verification steps

1. Add the programme/task/design records before changing any canonical reading path.
2. Update the canonical, planning, and workflow language together so they describe one pipeline.
3. Verify all new markdown links, plan/task references, and orientation paths.
4. Run `bash scripts/check-docs-gate.sh` and `git diff --check`.

## Completion checklist

- [x] PLAN-203 defines the single execution path, entry-point parity, integration gates, and
      release-oriented runnability matrix.
- [x] PLAN-202, the CPS calculus, Canonical Core, and target operational semantics agree on the
      calculus-to-CPS relationship.
- [x] AGENTS, PLAN-INDEX, the coverage map, and SPEC-INDEX route future work through PLAN-203
      without assigning downstream implementation to upstream feature tasks.
- [x] The traceability/proof policy explicitly makes experimental Verus obligations non-blocking.
- [x] CHANGELOG and documentation gates are updated and pass.

## Completion evidence

- `python3 tools/docs/validate_ash_cps_calculus.py --artifact docs/spec/ASH-CPS-CALCULUS.json --format json`
- `python3 tools/docs/validate_canonical_corpus.py --root . --manifest docs/spec/CANONICAL-CORPUS.json --format json --check-reference-frontmatter --require-promotion-completeness`
- `python3 tools/docs/validate_orientation_indexes.py --self-test`
- `bash scripts/check-docs-gate.sh`
- `bash scripts/check-semantic-task-gate-tests.sh`
- `git diff --check`
