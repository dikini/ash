# TASK-1395: Add metadata frontmatter to `reference/stdlib/algebra.md`

## Status: ✅ Complete

## Description

Add YAML metadata frontmatter to `reference/stdlib/algebra.md` matching the pattern of other stdlib reference pages (act, proc, workflow, result).

## Specification Reference

- [PLAN-139: Reference Maintenance and Staleness Remediation](../PLAN-139-REFERENCE-MAINTENANCE-AND-STALENESS-REMEDIATION.md)

## Dependencies

- Phase 138 complete (source content exists and is verified).

## Requirements

### Functional Requirements

- Add YAML frontmatter with `id: ref.stdlib.algebra`, `kind: reference`, `authority: canonical-adjacent`.
- `verified_against.git_commit` must point to the commit that added/verified the Phase 138 algebra content.
- `verified_against.specs` must link SPEC-078, SPEC-079, DESIGN-NOTE-INTERFACE-LAWS.
- `verified_against.tasks` must link TASK-1388 through TASK-1394.
- `verified_against.code` must link all `std/src/algebra/*.ash` and `std/src/option.ash`, `std/src/result.ash`.
- `verified_against.tests` must link parser law/proof tests.
- `refresh_trigger` must list all upstream files that should trigger a re-verify.

## Files

- Modify: `reference/stdlib/algebra.md`

## Verification

- [ ] Frontmatter parses as valid YAML.
- [ ] `id` is unique across reference corpus.
- [ ] All `verified_against` links resolve to existing files.
- [ ] `refresh_trigger` covers all upstream code/spec/task changes.
- [ ] Markdown link check passes.
