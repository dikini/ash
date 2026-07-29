# TASK-2045: Language Reference Placement, Authority, and Skeleton

**Status:** Planned
**Phase:** [PLAN-206](../PLAN-206-IMPLEMENTATION-BACKED-LANGUAGE-REFERENCE.md)
**Depends on:** TASK-2044
**Owned feature IDs:** Infrastructure only; no AUDIT-206 `LANG-*` feature row.

## Description

Reconcile the requested `docs/reference/language/` root with the legacy top-level `reference/`
corpus and cross-cutting `docs/reference/` contracts, then create the manual's navigable skeleton.

## Requirements

- Treat SPEC-071 §3.2 as requiring the reference corpus at top-level `reference/` unless
  superseded. Before creating a `docs/reference/language/` page, establish precisely one
  authority-approved outcome: (a) a SPEC, design, or policy amendment/supersession allowing that
  requested root, including every required index/policy update; or (b) a classification of
  `docs/reference/language/` as a separate non-SPEC-071 working/manual surface, including its
  authority and maintenance rules. Otherwise stop before creating skeleton pages; a task note is
  not authority to bypass the policy.
- Only after that outcome, create `docs/reference/language/index.md`, `status.md`,
  `source-of-truth.md`, and `conventions.md`; update `docs/README.md` to link the manual once
  present.
- Give each skeleton page an implementation/evidence/parity status convention, revision anchor,
  source/test evidence fields, and a rule that removed forms are never current examples.
- Decide whether a small manual-index/fence validator is needed; do not silently rely on the
  top-level `reference/` validator, which does not scan this directory.

## Handoffs and dependencies

- **Consumes:** PLAN-206 placement policy, SPEC-071 §3.2, AUDIT-206 conflicts,
  `reference/authority.md`, `tools/reference/check_frontmatter.py`, and
  `scripts/check-docs-gate.sh`.
- **Produces:** the manual root, status/source map convention, and stable destinations for
  TASK-2046 through TASK-2053.
- **Downstream owner:** TASK-2054 integrates navigation and any validator decision.
- **Non-goals:** migrating `reference/**`, making old references pass validation, or authoring
  domain pages beyond the skeleton.

## TDD and verification steps

1. Write an acceptance checklist asserting every skeleton page is linked from the language index
   and every page distinguishes implementation/evidence/parity.
2. Compare the requested placement with SPEC-071 §3.2 and the top-level `reference/` policy.
   Record precisely one authority-approved outcome: an amendment/supersession with its required
   index/policy updates, or a separate non-SPEC-071 working/manual classification with authority
   and maintenance rules. If neither outcome exists, record the blocker and stop.
3. Only after that outcome, create the skeleton and deliberately check a missing-link case before
   repairing it.
4. Run changed-Markdown link checks through `bash scripts/check-docs-gate.sh` and record any
   unrelated gate outcome.

## Completion checklist

- [ ] Placement reconciliation records exactly one authority-approved outcome—an
      amendment/supersession with required updates, or a separate-surface classification with
      authority/maintenance rules—and is linked from `source-of-truth.md`.
- [ ] The four root pages exist and are navigable from `docs/README.md`.
- [ ] No legacy form appears as a copyable current source example.
- [ ] Validator decision, status vocabulary, and refresh policy are documented.
- [ ] CHANGELOG and PLAN-INDEX are updated.
