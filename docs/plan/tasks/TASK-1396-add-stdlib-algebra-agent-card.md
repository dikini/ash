# TASK-1396: Add `stdlib-algebra.md` agent card

## Status: ✅ Complete

## Description

Create the missing agent derivative card `reference/agents/cards/stdlib-algebra.md` for `ref.stdlib.algebra`.

## Specification Reference

- [PLAN-139: Reference Maintenance and Staleness Remediation](../PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md)

## Dependencies

- TASK-1395 (frontmatter exists on canonical page).

## Requirements

### Functional Requirements

- Card follows the exact pattern of `stdlib-act.md`, `stdlib-proc.md`, `stdlib-workflow.md`, `stdlib-result.md`.
- `canonical_page: ref.stdlib.algebra`
- `canonical_page_path: ../../stdlib/algebra.md`
- `dependency_order: stdlib-algebra`
- Retrieval tags cover all algebra-related search terms.
- Stale-claim warnings prevent common overclaims.
- Edit preflight lists exact test files to run before editing.

## Files

- Create: `reference/agents/cards/stdlib-algebra.md`

## Verification

- [ ] Card frontmatter parses as valid YAML.
- [ ] `id` is unique across agent cards.
- [ ] `canonical_page` matches `ref.stdlib.algebra`.
- [ ] All `verified_against` links resolve to existing files.
- [ ] Markdown link check passes.
