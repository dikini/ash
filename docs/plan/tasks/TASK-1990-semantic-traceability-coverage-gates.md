# TASK-1990: Semantic Traceability and Coverage Gates

**Status:** Planned
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

- [ ] Stable node/edge schema is documented.
- [ ] Bidirectional coverage reports are reproducible.
- [ ] Assumptions and gaps remain visible.
- [ ] Canonical and implementation orphans fail closed.
