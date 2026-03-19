# Non-Lean Task Consistency Audit Design

## Goal

Create a companion audit report for non-Lean task documents in `docs/plan/tasks/`, using the earlier specification consistency audit as context for identifying task-level drift before reviewing the Rust codebase.

## Scope

- Review task documents for Rust-facing work only
- Exclude Lean-related tasks
- Focus on consistency between:
  - task docs,
  - referenced specs,
  - `docs/plan/PLAN-INDEX.md`,
  - the prior audit in [docs/audit/2026-03-19-spec-001-018-consistency-review.md](../audit/2026-03-19-spec-001-018-consistency-review.md)
- Produce a report in `docs/audit/`

## Output

Create a companion report at [docs/audit/2026-03-19-task-consistency-review-non-lean.md](../audit/2026-03-19-task-consistency-review-non-lean.md) with these sections:
- scope and exclusions
- relationship to prior spec audit
- high-severity findings
- medium-severity findings
- low-severity/systemic findings
- consistent task clusters
- implications for upcoming Rust code review

## Design Notes

- Keep the report separate from the earlier spec audit so task-level issues remain distinct from spec-level issues.
- Explicitly tie task inconsistencies back to already-known spec drift where relevant.
- Emphasize clusters with the highest review risk: policies, REPL/CLI, streams/runtime verification, and ADTs.
- Preserve a reporting-only posture; do not change task or spec files.
