# TASK-1990: Semantic Traceability and Coverage Gates

**Status:** Complete
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1985 and TASK-1989

## Description

Implement the stable REQ/GRAM/TYPE/LOWER/SEM/OBS/CONF/IMPL/TEST/PROOF graph and generate
bidirectional specification, implementation, test, and proof coverage reports.

## Requirements

- Validate controlled edge kinds and stable anchors.
- Record proof provider/tool/assumption/model/implementation fingerprints.
- Represent specified, implemented, tested, modelled, proved, assumed, deferred, refuted, and
  not-applicable separately.
- Fail on orphaned public semantic implementation and unowned canonical rules.

## TDD Steps

1. Add failing graph fixtures for dangling nodes, invalid edges, orphans, and false proof status.
2. Implement graph validation and report generation.
3. Seed the graph from the canonical calculus and two pilot targets.
4. Integrate appropriate checks into the docs/local gates.

## Completion Checklist

- [x] Stable node/edge schema is documented.
- [x] Bidirectional coverage reports are reproducible.
- [x] Assumptions and gaps remain visible.
- [x] Canonical and implementation orphans fail closed.

## Completion evidence

- `docs/spec/SEMANTIC-TRACEABILITY.json` contains the eight canonical-core owners, all frozen
  `λAsh-CPS` rule identities, two named implementation pilots, and explicit dispositions for work
  not yet realized. It does not make prototype Rust behavior normative.
- `TYPE-TARGET-ROW-001` records the executed row-normalization idempotence test and a deferred
  Verus obligation. `SEM-EFFECT-LOOKUP-001` records the current lookup implementation but leaves
  both the targeted test and the Rust-to-model proof visible as deferred/assumed TASK-1993 work.
- `docs/plan/audits/TASK-1990-semantic-traceability/specification-coverage.json` and
  `docs/plan/audits/TASK-1990-semantic-traceability/implementation-coverage.json` are generated
  deterministically by `tools/docs/validate_semantic_traceability.py`.
- The validator rejects dangling endpoints, uncontrolled edges, unstable anchors, false proof
  status, unowned canonical rules, and orphaned public semantic implementations. The docs gate
  invokes it on the repository artifact.
