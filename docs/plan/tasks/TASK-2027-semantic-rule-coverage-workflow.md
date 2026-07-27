# TASK-2027: Semantic-Rule-First Coverage Workflow

**Status:** Complete
**Phase:** Follow-up from TASK-1988 and TASK-1990

## Description

Make canonical semantic rules, rather than named source examples, the mandatory planning and
review unit for semantic implementation work.

## Requirements

- Add a canonical human-review coverage map linked to target specs and traceability.
- Require semantic tasks to declare a rule, domain, layer coverage, evidence, non-goals, and next
  obligation before implementation.
- Preserve TDD and traceability: examples are evidence only.
- Require bounded work to be labelled consistently in task, changelog, and traceability records.

## Completion evidence

- `AGENTS.md` requires the semantic-rule-first chain for semantic work.
- `docs/plan/SEMANTIC-RULE-COVERAGE.md` records current rule families and declared gaps.
- PLAN-INDEX and CHANGELOG direct future work to the map.
- Documentation and orientation gates pass.
