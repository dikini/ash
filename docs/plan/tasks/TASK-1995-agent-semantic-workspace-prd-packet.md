# TASK-1995: Agent Semantic Workspace PRD Packet

**Status:** Complete
**Track:** Incubating Agent Semantic Workspace

## Description

Move the exploratory Agent Semantic Workspace PRD into the Ash documentation tree and add the
accepted product-direction addendum that defines Ash dogfooding, unified CLI/harness operations,
and daemon integration roles. This task records design material only; it does not implement the
workspace or change Ash semantics.

## Requirements

- Store the original PRD under `docs/workspace/` without changing its requirements.
- Add a standalone addendum that records the accepted Ash implementation/dogfooding direction.
- State the Ash/workspace ownership boundary, promotion loop for Ash features, unified command
  model, CLI/daemon roles, and cross-repository coordination model.
- Add an orientation page that prevents the exploratory material from being mistaken for a
  normative Ash specification or implementation commitment.
- Index the completed documentation packet and record it in `CHANGELOG.md`.

## TDD Steps

1. Confirm the source PRD is present before moving it into the documentation tree.
2. Add the addendum and verify that it captures each decision made in the architecture discussion.
3. Add local and plan-index navigation links.
4. Run documentation link and orientation-index validation.

## Completion Checklist

- [x] The PRD is located at `docs/workspace/agent-semantic-workspace-prd.md`.
- [x] The accepted addendum is located beside the PRD.
- [x] The directory README states status and document authority.
- [x] PLAN-INDEX and CHANGELOG record the documentation packet.
- [x] Documentation validation has passed.

## Evidence

Validated on 2026-07-23:

```text
python3 tools/docs/validate_orientation_indexes.py --self-test
orientation-index-check: OK

bash scripts/check-docs-gate.sh
docs-gate: markdown links checked=1547 missing=0
orientation-index-check: OK
docs-gate: OK

git diff --check
exit 0
```
