# TASK-2027: Semantic-Rule-First Coverage Workflow

**Status:** Complete
**Semantic task classification:** non-semantic-workflow-enforcement
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
- Define `bounded`, `general`, `not applicable`, and `non-authorizing` as scoped
  implementation-domain/layer ownership, not as whole-language completion labels.
- Require new or materially revised semantic records to state their handoffs so a downstream
  owner, rather than an inappropriate cross-layer expansion, realizes each intentionally unowned
  layer.
- Keep end-to-end integration and refinement/proof evidence separate from individual
  feature/layer ownership.

## Completion evidence

- `AGENTS.md` requires the semantic-rule-first chain for semantic work.
- `docs/plan/SEMANTIC-RULE-COVERAGE.md` records current rule families and declared gaps.
- The workflow and coverage map explain how scoped rule-family handoffs compose without treating
  `not applicable` layers as missing behavior.
- PLAN-INDEX and CHANGELOG direct future work to the map.
- Documentation and orientation gates pass.
