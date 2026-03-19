# Rust Codebase Review Checklist Design

## Goal

Create a Rust-codebase review checklist mapped to the risky task clusters identified in the spec and task consistency audits.

## Scope

- Reporting and review guidance only
- Focus on Rust implementation review
- Map review work to the drift-heavy clusters identified in:
  - [docs/audit/2026-03-19-spec-001-018-consistency-review.md](../audit/2026-03-19-spec-001-018-consistency-review.md)
  - [docs/audit/2026-03-19-task-consistency-review-non-lean.md](../audit/2026-03-19-task-consistency-review-non-lean.md)
- Emphasize practical inspection targets in `ash-core`, `ash-parser`, `ash-typeck`, `ash-interp`, `ash-engine`, `ash-cli`, and `ash-repl`

## Output

Create a new checklist at [docs/audit/2026-03-19-rust-codebase-review-checklist.md](../audit/2026-03-19-rust-codebase-review-checklist.md) with:
- purpose and scope
- baseline review pass
- cluster-based review checklists
- cross-cutting verification items
- recommended review order

## Design Notes

- Prefer cluster-based sections over task-by-task sections so the checklist stays useful during live code review.
- Tie each cluster back to the task/spec drift already recorded in the audits.
- Include concrete file targets where available so the checklist can be used directly in-editor.
- Keep the checklist concise and action-oriented.
