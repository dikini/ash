# TASK-2029: Compositional Semantic Workflow Boundaries

**Status:** Complete — compositional implementation-domain ownership, named handoffs, and the
separate integration/proof boundary are recorded in the semantic workflow without changing
semantic behavior or task-record tooling.
**Semantic task classification:** non-semantic-workflow-enforcement
**Phase:** Follow-up from TASK-2027, TASK-2028, and the TASK-1988 implementation follow-ups

## Description

Make the semantic-rule workflow's implementation-domain boundaries explicit so future work treats
feature/layer records as composable handoffs. A `bounded` task owns its deliberate finite domain;
an intentionally unowned layer belongs to its named downstream owner rather than representing
missing behavior in the upstream task. End-to-end integration and refinement/proof work evaluate
the composition separately.

## Requirements

- Define `bounded`, `general`, `not applicable`, and `non-authorizing` as feature/domain and layer
  ownership terms in the agent workflow and coverage-map read path.
- Require new or materially revised semantic tasks to name consumed and produced handoffs, the
  owner of intentionally unowned downstream layers, and the separately owned integration/proof
  responsibility.
- Explain the TASK-2013 typed-handler facts → TASK-2014 admission/frame authorization → TASK-2008
  terminal-projection handoff without changing any current rule-family facts.
- Clarify that TASK-2027 and TASK-2028 establish scoped compositional ownership and evidence
  consistency, not whole-language execution completeness.
- Keep this as documentation/workflow policy only: do not change semantic specifications, Rust,
  JSON task-record domains, or validator behavior.

## Handoffs

- **Consumes:** TASK-2027's semantic-rule coverage workflow and TASK-2028's checked task-record
  evidence policy.
- **Produces:** the agent-facing ownership vocabulary and named-handoff read path used by future
  semantic tasks.
- **Does not own:** machine-readable handoff fields or validator enforcement. A later tooling task
  may validate named downstream owners after their record schema is deliberately designed.
- **Integration/proof responsibility:** separately owned end-to-end integration and refinement
  work validates composed task outputs; it does not change this workflow-policy task's scope.

## TDD and verification steps

1. Identify the ambiguous workflow wording that treats a bounded or unowned layer as a missing
   implementation claim.
2. Add the compositional ownership and named-handoff policy before modifying any rule-family facts.
3. Review the coverage map to confirm the explanatory section preserves every existing family,
   layer status, and next obligation.
4. Run `bash scripts/check-docs-gate.sh` and `git diff --check`.

## Completion checklist

- [x] `AGENTS.md` states compositional semantic ownership and the required handoff fields.
- [x] `SEMANTIC-RULE-COVERAGE.md` has an explanatory composition read path and the concrete
      TASK-2013 → TASK-2014 → TASK-2008 example.
- [x] TASK-2027, TASK-2028, PLAN-INDEX, and CHANGELOG use the scoped-composition interpretation.
- [x] Documentation validation and whitespace checks pass.
- [x] The deferred tooling boundary for mechanically validating named downstream owners is recorded
      without changing the semantic-record schema or validator.
