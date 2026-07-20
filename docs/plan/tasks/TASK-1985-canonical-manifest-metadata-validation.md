# TASK-1985: Canonical Manifest, Metadata, and Validation

**Status:** Planned
**Phase:** [PLAN-202](../PLAN-202-FORMAL-SEMANTICS-AND-VERIFICATION-PROGRAMME.md)
**Depends on:** TASK-1984

## Description

Define and implement the A0-A5 authority manifest and metadata validation needed to make semantic
ownership, supersession, dependencies, and derived context packs machine-checkable.

## Requirements

- Define a versioned `canonical-corpus/v1` sidecar schema with `canonical_for`, authority level,
  controlled lifecycle, stable trace-node support, and inherited SPEC-071 evidence/relationship
  concepts.
- Keep SPEC-071 `reference/` frontmatter enums and required fields valid; document typed edges
  between the separate canonical and reference schemas instead of reusing incompatible enums.
- Validate unique ids/owners, acyclic supersession, valid paths, controlled enums, and generated
  artifact freshness.
- Keep top-level `reference/` derivative and preserve git-backed snapshot manifests.
- Reject authority conflicts rather than selecting a winner implicitly.

## TDD Steps

1. Add failing validator self-tests for duplicate ownership, supersession cycles, broken sources,
   and derivative authority leakage.
2. Implement the schema and validator changes.
3. Add the initial manifest entries without promoting unresolved conflicts.
4. Run validator self-tests and documentation gates.

## Completion Checklist

- [ ] Manifest/schema is documented and validated.
- [ ] Existing reference metadata remains compatible or has an explicit migration.
- [ ] Conflict and cycle fixtures fail closed.
- [ ] Generated context packs cannot become authority.
